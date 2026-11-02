use crate::{
    commands::{AppState, with_engine},
    packages::PackageState,
};
use aevrik_engine::npm::{NodeRuntime, NpmServiceInput, ProjectScripts, display_path, npm_cli};
use std::{path::Path, sync::Arc};
use tauri::State;

#[tauri::command]
pub async fn project_scripts(
    state: State<'_, AppState>,
    packages: State<'_, PackageState>,
    id: String,
) -> Result<ProjectScripts, String> {
    let manager = Arc::clone(&packages.0);
    with_engine(state, move |engine| {
        let mut info = engine.project_scripts(&id).map_err(|e| e.to_string())?;
        for package in manager.list().map_err(|e| e.to_string())? {
            if package.package.family != "node" || package.status != "installed" {
                continue;
            }
            if let Some(path) = package.path {
                let node = Path::new(&path);
                if npm_cli(node).is_ok() {
                    let executable = display_path(node);
                    if let Some(existing) = info
                        .nodes
                        .iter_mut()
                        .find(|n| n.executable.eq_ignore_ascii_case(&executable))
                    {
                        existing.label = format!("Node.js {} · on PATH", package.package.version);
                    } else {
                        info.nodes.push(NodeRuntime {
                            executable,
                            label: format!("Node.js {}", package.package.version),
                        });
                    }
                }
            }
        }
        Ok(info)
    })
    .await
}

#[tauri::command]
pub async fn register_npm_service(
    state: State<'_, AppState>,
    input: NpmServiceInput,
) -> Result<(), String> {
    with_engine(state, move |engine| {
        engine
            .register_npm_service(input)
            .map(|_| ())
            .map_err(|e| e.to_string())
    })
    .await
}
