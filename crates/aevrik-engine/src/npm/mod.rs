//! Read package scripts and launch bundled npm with a selected Node runtime.
use crate::{Engine, Result, Service, ServiceInput, error::invalid};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    io::Read,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, Serialize)]
pub struct Script {
    pub name: String,
    pub command: String,
}

#[derive(Serialize)]
pub struct NodeRuntime {
    pub executable: String,
    pub label: String,
}

#[derive(Serialize)]
pub struct ProjectScripts {
    pub scripts: Vec<Script>,
    pub nodes: Vec<NodeRuntime>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NpmServiceInput {
    pub project_id: String,
    pub node_executable: String,
    pub script: String,
    pub port: Option<u16>,
}

pub fn scripts(directory: &Path) -> Result<Vec<Script>> {
    let file = std::fs::File::open(directory.join("package.json"))
        .map_err(|e| invalid(format!("Cannot read package.json in this project: {e}")))?;
    let mut bytes = Vec::new();
    file.take(1_048_577).read_to_end(&mut bytes)?;
    if bytes.len() > 1_048_576 {
        return Err(invalid("package.json exceeds the 1 MiB limit."));
    }
    #[derive(Deserialize)]
    struct Manifest {
        #[serde(default)]
        scripts: BTreeMap<String, String>,
    }
    let manifest: Manifest = serde_json::from_slice(&bytes)
        .map_err(|e| invalid(format!("Invalid package.json scripts: {e}")))?;
    Ok(manifest
        .scripts
        .into_iter()
        .map(|(name, command)| Script { name, command })
        .collect())
}

// cmd.exe (used by npm internally) does not understand extended-length path prefixes.
pub fn display_path(path: &Path) -> String {
    let text = path.to_string_lossy();
    if let Some(unc) = text.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{unc}")
    } else {
        text.trim_start_matches(r"\\?\").into()
    }
}

pub fn npm_cli(node: &Path) -> Result<PathBuf> {
    if !node.is_absolute()
        || !node.is_file()
        || !node
            .file_name()
            .is_some_and(|n| n.eq_ignore_ascii_case("node.exe"))
    {
        return Err(invalid("Choose an existing node.exe using its full path."));
    }
    let cli = node
        .parent()
        .ok_or_else(|| invalid("Node folder is unavailable."))?
        .join("node_modules/npm/bin/npm-cli.js");
    if !cli.is_file() {
        return Err(invalid(
            "This Node installation does not include npm. Choose a Node.js runtime with bundled npm.",
        ));
    }
    Ok(cli.canonicalize()?)
}

pub(crate) fn validate(config: &mut ServiceInput) -> Result<()> {
    let Some(script) = &config.npm_script else {
        return Ok(());
    };
    if script.is_empty() || script.len() > 256 || script.chars().any(char::is_control) {
        return Err(invalid("Choose a valid npm script."));
    }
    let entries = scripts(Path::new(&config.cwd))?;
    if !entries
        .iter()
        .any(|s| s.name == *script && !s.command.trim().is_empty())
    {
        return Err(invalid(
            "The selected npm script is missing or empty. Reload the project scripts.",
        ));
    }
    let cli = npm_cli(Path::new(&config.executable))?;
    config.args = vec![
        display_path(&cli),
        "run".into(),
        "--".into(),
        script.clone(),
    ];
    config.executable = display_path(Path::new(&config.executable));
    config.cwd = display_path(Path::new(&config.cwd));
    Ok(())
}

pub(crate) fn environment(command: &mut Command, config: &ServiceInput) -> Result<()> {
    if config.npm_script.is_none() {
        return Ok(());
    }
    let parent = Path::new(&config.executable)
        .parent()
        .ok_or_else(|| invalid("Node folder is unavailable."))?;
    let mut paths = vec![parent.to_path_buf()];
    if let Some(existing) = std::env::var_os("PATH") {
        paths.extend(std::env::split_paths(&existing));
    }
    let path = std::env::join_paths(paths)
        .map_err(|_| invalid("The selected Node folder cannot be represented in PATH."))?;
    command.env("PATH", path);
    Ok(())
}

impl Engine {
    pub fn project_scripts(&self, id: &str) -> Result<ProjectScripts> {
        let project = self
            .store
            .projects()?
            .into_iter()
            .find(|p| p.id == id)
            .ok_or_else(|| invalid("Project no longer exists."))?;
        let mut nodes = Vec::new();
        if let Some(path) = std::env::var_os("PATH") {
            for dir in std::env::split_paths(&path).filter(|p| p.is_absolute()) {
                let node = dir.join("node.exe");
                if npm_cli(&node).is_ok() {
                    nodes.push(NodeRuntime {
                        executable: display_path(&node),
                        label: "Node.js on PATH".into(),
                    });
                    break;
                }
            }
        }
        Ok(ProjectScripts {
            scripts: scripts(Path::new(&project.path))?,
            nodes,
        })
    }

    pub fn register_npm_service(&mut self, input: NpmServiceInput) -> Result<Service> {
        let project = self
            .store
            .projects()?
            .into_iter()
            .find(|p| p.id == input.project_id)
            .ok_or_else(|| invalid("Project no longer exists."))?;
        let mut config = ServiceInput {
            name: format!("{} · {}", project.name, input.script)
                .chars()
                .take(120)
                .collect(),
            executable: input.node_executable,
            args: Vec::new(),
            cwd: project.path.clone(),
            port: input.port,
            npm_script: Some(input.script),
        };
        crate::validation::service(&mut config)?;
        self.store.add_project_service(&input.project_id, config)
    }
}
