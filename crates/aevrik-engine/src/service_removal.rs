use crate::{Engine, Result, error::invalid};
use std::path::{Path, PathBuf};

impl Engine {
    pub fn remove_service(&mut self, id: &str) -> Result<()> {
        self.processes.poll()?;
        let mut service = self.store.service(id)?;
        self.processes.decorate(&mut service);
        if service.state == "running" {
            return Err(invalid(
                "Stop this service before removing its registration.",
            ));
        }
        self.store.remove_service(id)
    }
    pub fn protected_runtime_paths(&self) -> Result<Vec<PathBuf>> {
        let mut paths = Vec::new();
        for service in self.store.services()? {
            paths.push(PathBuf::from(&service.config.executable));
            paths.push(PathBuf::from(&service.config.cwd));
            for argument in &service.config.args {
                let value = argument
                    .split_once('=')
                    .map_or(argument.as_str(), |(_, value)| value);
                if Path::new(value).is_absolute() {
                    paths.push(PathBuf::from(value));
                }
            }
        }
        paths.extend(
            self.store
                .projects()?
                .iter()
                .map(|p| PathBuf::from(&p.path)),
        );
        Ok(paths)
    }
}
