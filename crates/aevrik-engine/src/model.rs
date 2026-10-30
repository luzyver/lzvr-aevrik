use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectInput {
    pub name: String,
    pub path: String,
    pub domain: String,
    pub runtime: String,
    pub service_id: Option<String>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    #[serde(flatten)]
    pub config: ProjectInput,
    pub state: String,
}
impl std::ops::Deref for Project {
    type Target = ProjectInput;
    fn deref(&self) -> &Self::Target {
        &self.config
    }
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceInput {
    pub name: String,
    pub executable: String,
    pub args: Vec<String>,
    pub cwd: String,
    pub port: Option<u16>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    pub id: String,
    #[serde(flatten)]
    pub config: ServiceInput,
    pub state: String,
    pub pid: Option<u32>,
    pub exit_code: Option<i32>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub timestamp: u64,
    pub service: String,
    pub level: String,
    pub message: String,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub projects: Vec<Project>,
    pub services: Vec<Service>,
    pub logs: Vec<LogEntry>,
    pub theme: String,
}
