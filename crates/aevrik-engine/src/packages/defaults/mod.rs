//! Explicit Windows terminal defaults. Installing a package never edits PATH.
mod environment;
#[cfg(windows)]
mod registry;
#[cfg(test)]
mod tests;

use super::{PackageError, PackageManager, PackageResult, archive};
use environment::Environment;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    User,
    Machine,
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DefaultStatus {
    supported: bool,
    active: bool,
    on_path: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DefaultPlan {
    scope: Scope,
    removing: bool,
    executable: String,
    current_executable: Option<String>,
    directories: Vec<String>,
    fingerprint: String,
}

fn invalid(message: &str) -> PackageError {
    PackageError::Invalid(message.into())
}

// These CLI distributions do not require changing JAVA_HOME, GOROOT or SDK resolver policy.
fn command(family: &str) -> Option<&'static str> {
    match family {
        "node" => Some("node"),
        "python" => Some("python"),
        "php" => Some("php"),
        "bun" => Some("bun"),
        "deno" => Some("deno"),
        "zig" => Some("zig"),
        "nim" => Some("nim"),
        _ => None,
    }
}

fn key(path: &str) -> String {
    let path = path.trim().trim_matches('"');
    let canonical = Path::new(path).canonicalize().ok();
    display_path(
        &canonical
            .as_deref()
            .unwrap_or(Path::new(path))
            .to_string_lossy(),
    )
    .replace('/', "\\")
    .trim_end_matches('\\')
    .to_lowercase()
}

fn display_path(path: &str) -> String {
    if let Some(unc) = path.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{unc}")
    } else {
        path.trim_start_matches(r"\\?\").to_owned()
    }
}

fn directories(executable: &Path, family: &str) -> PackageResult<Vec<String>> {
    let parent = executable
        .parent()
        .ok_or_else(|| invalid("Runtime has no executable folder."))?;
    let mut paths = vec![parent.to_owned()];
    if family == "python" {
        paths.push(parent.join("Scripts"));
    }
    paths.into_iter().map(|p| {
        let value = display_path(p.to_str().ok_or_else(|| invalid("Runtime path is not valid Unicode."))?);
        if value.contains([';', '%', '"', '\0', '\n', '\r']) {
            return Err(invalid("This runtime folder cannot be represented safely in Windows PATH. Install it in a folder without semicolons, percent signs or quotes."));
        }
        Ok(value)
    }).collect()
}

fn edit_path(
    raw: &str,
    dirs: &[String],
    removing: bool,
    expand: impl Fn(&str) -> String,
) -> String {
    let mut parts: Vec<String> = if removing { vec![] } else { dirs.to_vec() };
    if !raw.is_empty() {
        parts.extend(
            raw.split(';')
                .filter(|entry| !dirs.iter().any(|dir| key(dir) == key(&expand(entry))))
                .map(str::to_owned),
        );
    }
    parts.join(";")
}

fn choose_scope(removing: bool, machine_command: bool, machine_entry: bool) -> Scope {
    if (removing && machine_entry) || (!removing && machine_command) {
        Scope::Machine
    } else {
        Scope::User
    }
}

pub(super) struct DefaultsSnapshot(Environment);
pub(super) struct PathChangeLock {
    #[cfg(windows)]
    _guard: registry::PathLock,
}
impl PathChangeLock {
    pub(super) fn acquire() -> PackageResult<Self> {
        Ok(Self {
            #[cfg(windows)]
            _guard: registry::PathLock::acquire()?,
        })
    }
}
impl DefaultsSnapshot {
    pub(super) fn read() -> PackageResult<Self> {
        Ok(Self(Environment::read()?))
    }
    pub(super) fn status(&self, family: &str, executable: Option<&Path>) -> DefaultStatus {
        let Some(cmd) = command(family).filter(|_| cfg!(windows)) else {
            return DefaultStatus::default();
        };
        let Some(exe) = executable else {
            return DefaultStatus {
                supported: true,
                ..DefaultStatus::default()
            };
        };
        let dirs = directories(exe, family).unwrap_or_default();
        DefaultStatus {
            supported: true,
            active: self
                .0
                .first(cmd, None)
                .is_some_and(|p| key(&p) == key(&exe.to_string_lossy())),
            on_path: self.0.contains(&dirs, Scope::Machine) || self.0.contains(&dirs, Scope::User),
        }
    }
    pub(super) fn protect(&self, directory: &Path) -> PackageResult<()> {
        let target = key(&directory.to_string_lossy());
        let prefix = format!("{target}\\");
        if self
            .0
            .paths(None)
            .iter()
            .any(|p| key(p) == target || key(p).starts_with(&prefix))
        {
            return Err(invalid(
                "This runtime folder is still on Windows PATH. Use Remove from PATH before uninstalling, or remove its PATH entry in Windows Environment Variables.",
            ));
        }
        Ok(())
    }
}

impl PackageManager {
    fn default_target(&self, id: &str) -> PackageResult<(PathBuf, Vec<String>, &'static str)> {
        if !cfg!(windows) {
            return Err(invalid("Terminal defaults are supported on Windows only."));
        }
        let package = self
            .catalog()?
            .into_iter()
            .find(|p| p.id == id)
            .ok_or_else(|| invalid("Unknown catalog package."))?;
        let cmd = command(&package.family)
            .ok_or_else(|| invalid("This runtime does not support terminal defaults yet."))?;
        let root = self.package_root(id)?;
        let exe = archive::installed(&root, &package)
            .ok_or_else(|| invalid("Install this runtime before setting a default."))?;
        let dirs = directories(&exe, &package.family)?;
        Ok((exe, dirs, cmd))
    }

    fn default_plan(
        &self,
        id: &str,
        removing: bool,
        env: &Environment,
    ) -> PackageResult<DefaultPlan> {
        let (exe, dirs, cmd) = self.default_target(id)?;
        if !removing && !env.extensions().iter().any(|e| e == ".exe") {
            return Err(invalid(
                "Windows PATHEXT does not include .EXE. Restore .EXE in Windows Environment Variables before selecting a terminal default.",
            ));
        }
        let machine = env.first(cmd, Some(Scope::Machine));
        let scope = choose_scope(
            removing,
            machine.is_some(),
            env.contains(&dirs, Scope::Machine),
        );
        if removing && !env.contains(&dirs, scope) {
            return Err(invalid(
                "This runtime is no longer on PATH. Refresh the runtime list.",
            ));
        }
        let mut digest = Sha256::new();
        digest.update(serde_json::to_vec(env)?);
        digest.update(serde_json::to_vec(&(id, removing, scope, &dirs))?);
        Ok(DefaultPlan {
            scope,
            removing,
            executable: display_path(&exe.to_string_lossy()),
            current_executable: env.first(cmd, None),
            directories: dirs,
            fingerprint: format!("{:x}", digest.finalize()),
        })
    }

    pub fn preview_default(&self, id: &str, removing: bool) -> PackageResult<DefaultPlan> {
        let _lock = self.package_lock(id)?;
        self.default_plan(id, removing, &Environment::read()?)
    }

    pub fn apply_default(
        &self,
        id: &str,
        removing: bool,
        scope: Scope,
        fingerprint: &str,
    ) -> PackageResult<()> {
        // Shared with uninstall. Recompute from installed files and live registry; never trust UI paths.
        let _windows_lock = PathChangeLock::acquire()?;
        let _defaults_lock = self.package_lock("runtime-defaults")?;
        let _package_lock = self.package_lock(id)?;
        let env = Environment::read()?;
        let plan = self.default_plan(id, removing, &env)?;
        if plan.scope != scope || plan.fingerprint != fingerprint {
            return Err(invalid(
                "Windows PATH changed since this dialog opened. Close it and try again to review the new plan.",
            ));
        }
        let value = env.path(scope);
        let next = edit_path(&value.text, &plan.directories, removing, |p| {
            env.expand(p, scope, value.kind)
        });
        if removing {
            let mut remaining = env.path(scope);
            remaining.text = next.clone();
            if env
                .expand(&remaining.text, scope, remaining.kind)
                .split(';')
                .any(|p| plan.directories.iter().any(|d| key(p) == key(d)))
            {
                return Err(invalid(
                    "This runtime is included through a shared environment variable. Remove its folder from that variable in Windows Environment Variables before uninstalling.",
                ));
            }
        }
        if next.encode_utf16().count() >= 32760 {
            return Err(invalid(
                "Windows PATH is too long. Shorten it before selecting a default.",
            ));
        }
        // Save only PATH, never the rest of the environment (which can contain credentials).
        let backup = self
            .root
            .join(format!("path-backup-{}.json", uuid::Uuid::new_v4()));
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(backup)?;
        serde_json::to_writer_pretty(
            &mut file,
            &serde_json::json!({
                "scope": scope, "path": value.text, "registryKind": value.kind,
                "existed": if scope == Scope::Machine { env.machine.contains_key("PATH") } else { env.user.contains_key("PATH") }
            }),
        )?;
        file.sync_all()?;
        env.write_path(scope, &next)
    }
}
