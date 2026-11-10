//! Edit or remove saved registrations, control every service at once and
//! restore the services that were running when Aevrik last exited.
use crate::{Engine, Result, ServiceInput, error::invalid};

const RESTORE: &str = "restore_services";
const LAST_RUNNING: &str = "last_running";

impl Engine {
    pub fn remove_project(&mut self, id: &str) -> Result<()> {
        self.store.remove_project(id)
    }
    pub fn edit_project(
        &mut self,
        id: &str,
        name: String,
        path: String,
        domain: String,
    ) -> Result<()> {
        self.processes.poll()?;
        let linked = self
            .store
            .projects()?
            .into_iter()
            .find(|p| p.id == id)
            .and_then(|p| p.config.service_id)
            .and_then(|service| self.store.service(&service).ok())
            .filter(|service| service.config.npm_script.is_some());
        let running = linked
            .as_ref()
            .is_some_and(|service| self.processes.running().contains(&service.id));
        self.store
            .edit_project(id, name, path, domain, linked, running)
    }
    /// How many owned processes are alive right now.
    pub fn running_count(&mut self) -> Result<usize> {
        self.processes.poll()?;
        Ok(self.processes.running().len())
    }
    pub fn update_service(&mut self, id: &str, mut input: ServiceInput) -> Result<()> {
        self.processes.poll()?;
        let mut service = self.store.service(id)?;
        self.processes.decorate(&mut service);
        if service.state == "running" {
            return Err(invalid("Stop this service before editing it."));
        }
        // The npm script belongs to the project link; editing keeps it.
        input.npm_script = service.config.npm_script;
        crate::validation::service(&mut input)?;
        self.store.update_service(id, &input)
    }
    /// Starts every stopped service. Returns one message per service that failed.
    pub fn start_all(&mut self) -> Result<Vec<String>> {
        self.processes.poll()?;
        let running = self.processes.running();
        let mut failures = Vec::new();
        for service in self.store.services()? {
            if running.contains(&service.id) {
                continue;
            }
            if let Err(error) = self.processes.start(&service) {
                let message = format!("{}: {error}", service.config.name);
                self.processes.logs.push(&service.id, "ERROR", &message);
                failures.push(message);
            }
        }
        self.remember_running()?;
        Ok(failures)
    }
    pub fn stop_all(&mut self) -> Result<()> {
        let result = self.processes.shutdown();
        self.remember_running()?;
        result
    }
    pub fn restore_enabled(&self) -> Result<bool> {
        Ok(self.store.preference(RESTORE)?.as_deref() == Some("true"))
    }
    pub fn set_restore_services(&mut self, enabled: bool) -> Result<()> {
        self.store
            .set_preference(RESTORE, if enabled { "true" } else { "false" })?;
        self.remember_running()
    }
    /// Starts the services recorded at the last start or stop, when enabled.
    pub fn restore_services(&mut self) -> Result<()> {
        if !self.restore_enabled()? {
            return Ok(());
        }
        let saved = self.store.preference(LAST_RUNNING)?.unwrap_or_default();
        let ids: Vec<String> = serde_json::from_str(&saved).unwrap_or_default();
        for id in ids {
            // A removed registration or a busy port must not block the others.
            let Ok(service) = self.store.service(&id) else {
                continue;
            };
            if let Err(error) = self.processes.start(&service) {
                self.processes.logs.push(
                    &id,
                    "ERROR",
                    format!("Could not restore {}: {error}", service.config.name),
                );
            }
        }
        Ok(())
    }
    /// Records the processes that are still alive; one that exited by itself is not restored.
    pub(crate) fn remember_running(&mut self) -> Result<()> {
        self.processes.poll()?;
        let ids = serde_json::to_string(&self.processes.running())?;
        self.store.set_preference(LAST_RUNNING, &ids)
    }
}
