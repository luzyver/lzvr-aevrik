//! Local metadata and owned child-process management, independent of Tauri.
pub mod autostart;
pub mod data_location;
mod editing;
mod error;
mod job;
mod logs;
mod model;
pub mod npm;
mod process;
mod service_removal;
mod storage;
mod store;
pub mod updates;
mod validation;
pub use error::{Error, Result};
pub use model::*;
use std::path::Path;
pub use storage::data_directory;

pub struct Engine {
    store: store::Store,
    processes: process::Processes,
}
impl Engine {
    pub fn open(path: &Path) -> Result<Self> {
        Ok(Self {
            store: store::Store::open(path)?,
            processes: process::Processes::new(),
        })
    }
    pub fn add_project(&mut self, mut input: ProjectInput) -> Result<Project> {
        validation::project(&mut input)?;
        let project = self.store.add_project(input)?;
        self.processes.logs.push(
            "engine",
            "INFO",
            format!("Registered project {}.", project.name),
        );
        Ok(project)
    }
    pub fn register_service(&mut self, mut input: ServiceInput) -> Result<Service> {
        validation::service(&mut input)?;
        let service = self.store.add_service(input)?;
        self.processes.logs.push(
            "engine",
            "INFO",
            format!("Registered service {}.", service.config.name),
        );
        Ok(service)
    }
    pub fn snapshot(&mut self) -> Result<Snapshot> {
        let alive = self.processes.running().len();
        self.processes.poll()?;
        if self.processes.running().len() != alive {
            self.remember_running()?;
        }
        let mut services = self.store.services()?;
        for service in &mut services {
            self.processes.decorate(service);
        }
        let mut projects = self.store.projects()?;
        for project in &mut projects {
            if let Some(service) = services
                .iter()
                .find(|s| Some(&s.id) == project.service_id.as_ref())
            {
                project.state = service.state.clone();
            }
        }
        Ok(Snapshot {
            projects,
            services,
            logs: self.processes.logs.snapshot(),
            theme: self.store.theme()?,
            restore_services: self.restore_enabled()?,
        })
    }
    pub fn start_service(&mut self, id: &str) -> Result<()> {
        let service = self.store.service(id)?;
        self.processes.start(&service)?;
        self.remember_running()
    }
    pub fn stop_service(&mut self, id: &str) -> Result<()> {
        self.store.service(id)?;
        self.processes.stop(id)?;
        self.remember_running()
    }
    pub fn restart_service(&mut self, id: &str) -> Result<()> {
        self.stop_service(id)?;
        self.start_service(id)
    }
    pub fn set_theme(&mut self, theme: &str) -> Result<()> {
        self.store.set_theme(theme)
    }
    pub fn update_project(
        &mut self,
        id: &str,
        service_id: Option<String>,
        runtime: String,
    ) -> Result<()> {
        self.store.update_project(id, service_id, runtime)
    }
    pub fn clear_logs(&self) {
        self.processes.logs.clear();
    }
    pub fn shutdown(&mut self) -> Result<()> {
        // Save what is alive before stopping it; restore starts exactly these.
        let remembered = self.remember_running();
        let stopped = self.processes.shutdown();
        remembered.and(stopped)
    }
}
pub mod packages;
