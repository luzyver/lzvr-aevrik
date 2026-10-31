use super::{PackageResult, Scope, invalid, key};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct Value {
    pub text: String,
    pub kind: u32,
}
impl Default for Value {
    fn default() -> Self {
        Self {
            text: String::new(),
            kind: 2,
        }
    }
}

#[derive(Default, PartialEq, Eq, Serialize)]
pub(super) struct Environment {
    pub machine: BTreeMap<String, Value>,
    pub user: BTreeMap<String, Value>,
}
impl Environment {
    pub fn read() -> PackageResult<Self> {
        #[cfg(windows)]
        {
            super::registry::read()
        }
        #[cfg(not(windows))]
        {
            Ok(Self::default())
        }
    }
    pub fn path(&self, scope: Scope) -> Value {
        let map = if scope == Scope::Machine {
            &self.machine
        } else {
            &self.user
        };
        map.get("PATH").cloned().unwrap_or_default()
    }
    pub fn expand(&self, value: &str, scope: Scope, kind: u32) -> String {
        self.expand_inner(value, scope, kind, 0)
    }
    fn expand_inner(&self, value: &str, scope: Scope, kind: u32, depth: usize) -> String {
        if kind != 2 || depth >= 8 {
            return value.to_owned();
        }
        let mut next = String::new();
        let mut rest = value;
        while let Some(start) = rest.find('%') {
            next.push_str(&rest[..start]);
            let tail = &rest[start + 1..];
            let Some(end) = tail.find('%') else {
                next.push_str(&rest[start..]);
                rest = "";
                break;
            };
            let name = tail[..end].to_uppercase();
            // User PATH may refer to the system PATH; it must never expand itself recursively.
            let variable_scope = if name == "PATH" {
                Scope::Machine
            } else {
                scope
            };
            let found = if variable_scope == Scope::User {
                self.user.get(&name)
            } else {
                None
            }
            .or_else(|| self.machine.get(&name));
            let replacement = found
                .map(|v| self.expand_inner(&v.text, variable_scope, v.kind, depth + 1))
                .or_else(|| match name.as_str() {
                    // Profile/bootstrap variables are supplied by Windows, not these two registry keys.
                    "SYSTEMROOT"
                    | "SYSTEMDRIVE"
                    | "ALLUSERSPROFILE"
                    | "PUBLIC"
                    | "USERPROFILE"
                    | "HOMEDRIVE"
                    | "HOMEPATH"
                    | "APPDATA"
                    | "LOCALAPPDATA"
                    | "PROGRAMFILES"
                    | "PROGRAMFILES(X86)"
                    | "PROGRAMW6432"
                    | "COMMONPROGRAMFILES"
                    | "COMMONPROGRAMFILES(X86)"
                    | "COMMONPROGRAMW6432"
                    | "PROGRAMDATA" => std::env::var(&name).ok(),
                    _ => None,
                });
            next.push_str(&replacement.unwrap_or_else(|| format!("%{}%", &tail[..end])));
            rest = &tail[end + 1..];
        }
        next.push_str(rest);
        next
    }
    pub fn paths(&self, scope: Option<Scope>) -> Vec<String> {
        [Scope::Machine, Scope::User]
            .into_iter()
            .filter(|s| scope.is_none_or(|wanted| wanted == *s))
            .flat_map(|s| {
                let value = self.path(s);
                // Empty/relative entries depend on the shell's working directory, outside a global default.
                self.expand(&value.text, s, value.kind)
                    .split(';')
                    .map(|p| p.trim().trim_matches('"').to_owned())
                    .filter(|p| Path::new(p).is_absolute())
                    .collect::<Vec<_>>()
            })
            .collect()
    }
    pub fn first(&self, command: &str, scope: Option<Scope>) -> Option<String> {
        let extensions = self.extensions();
        self.paths(scope).into_iter().find_map(|folder| {
            extensions
                .iter()
                .map(|ext| Path::new(&folder).join(format!("{command}{ext}")))
                .find(|p| p.is_file())
                .map(|p| p.to_string_lossy().into_owned())
        })
    }
    pub fn extensions(&self) -> Vec<String> {
        self.user
            .get("PATHEXT")
            .or_else(|| self.machine.get("PATHEXT"))
            .map_or(
                ".COM;.EXE;.BAT;.CMD;.VBS;.VBE;.JS;.JSE;.WSF;.WSH;.MSC",
                |v| v.text.as_str(),
            )
            .split(';')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| {
                s.strip_prefix('.').is_some_and(|tail| {
                    !tail.is_empty() && tail.chars().all(|c| c.is_ascii_alphanumeric())
                })
            })
            .collect()
    }
    pub fn contains(&self, dirs: &[String], scope: Scope) -> bool {
        self.paths(Some(scope))
            .iter()
            .any(|p| dirs.iter().any(|dir| key(p) == key(dir)))
    }
    pub fn write_path(&self, scope: Scope, next: &str) -> PackageResult<()> {
        // Avoid overwriting an edit by Windows Settings or another Aevrik workspace.
        if &Self::read()? != self {
            return Err(invalid(
                "Windows environment changed. Close the dialog and try again.",
            ));
        }
        #[cfg(windows)]
        {
            super::registry::write_path(scope, self.path(scope).kind, next)
        }
        #[cfg(not(windows))]
        {
            let _ = (scope, next);
            Err(invalid("Terminal defaults require Windows."))
        }
    }
}
