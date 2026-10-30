use crate::{
    error::{Result, invalid},
    model::*,
};
use rusqlite::{Connection, OptionalExtension, params};
use std::path::Path;

pub(crate) struct Store {
    connection: Connection,
}
impl Store {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let connection = Connection::open(path)?;
        connection.busy_timeout(std::time::Duration::from_secs(3))?;
        connection.execute_batch("PRAGMA foreign_keys=ON;
          CREATE TABLE IF NOT EXISTS services(id TEXT PRIMARY KEY, config TEXT NOT NULL);
          CREATE TABLE IF NOT EXISTS projects(id TEXT PRIMARY KEY, name TEXT NOT NULL COLLATE NOCASE UNIQUE,
            domain TEXT NOT NULL COLLATE NOCASE UNIQUE, config TEXT NOT NULL);
          CREATE TABLE IF NOT EXISTS preferences(key TEXT PRIMARY KEY, value TEXT NOT NULL);")?;
        Ok(Self { connection })
    }
    pub fn add_project(&mut self, config: ProjectInput) -> Result<Project> {
        let exists: bool = self.connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM projects WHERE domain=?1 OR name=?2)",
            params![config.domain, config.name],
            |r| r.get(0),
        )?;
        if exists {
            return Err(invalid("A project already uses that name or domain."));
        }
        if let Some(id) = &config.service_id {
            self.service(id)?;
        }
        let id = uuid::Uuid::new_v4().to_string();
        self.connection.execute(
            "INSERT INTO projects(id,name,domain,config) VALUES(?1,?2,?3,?4)",
            params![
                id,
                config.name,
                config.domain,
                serde_json::to_string(&config)?
            ],
        )?;
        Ok(Project {
            id,
            config,
            state: "stopped".into(),
        })
    }
    pub fn add_service(&mut self, config: ServiceInput) -> Result<Service> {
        let id = uuid::Uuid::new_v4().to_string();
        self.connection.execute(
            "INSERT INTO services(id,config) VALUES(?1,?2)",
            params![id, serde_json::to_string(&config)?],
        )?;
        Ok(Service {
            id,
            config,
            state: "stopped".into(),
            pid: None,
            exit_code: None,
        })
    }
    pub fn service(&self, id: &str) -> Result<Service> {
        let config: Option<String> = self
            .connection
            .query_row("SELECT config FROM services WHERE id=?1", [id], |r| {
                r.get(0)
            })
            .optional()?;
        let config = config.ok_or_else(|| invalid("Service no longer exists."))?;
        Ok(Service {
            id: id.into(),
            config: serde_json::from_str(&config)?,
            state: "stopped".into(),
            pid: None,
            exit_code: None,
        })
    }
    pub fn remove_service(&mut self, id: &str) -> Result<()> {
        let projects = self.projects()?;
        let transaction = self.connection.transaction()?;
        for mut project in projects {
            if project.config.service_id.as_deref() == Some(id) {
                project.config.service_id = None;
                transaction.execute(
                    "UPDATE projects SET config=?1 WHERE id=?2",
                    params![serde_json::to_string(&project.config)?, project.id],
                )?;
            }
        }
        transaction.execute("DELETE FROM services WHERE id=?1", [id])?;
        transaction.commit()?;
        Ok(())
    }
    pub fn services(&self) -> Result<Vec<Service>> {
        let mut query = self
            .connection
            .prepare("SELECT id FROM services ORDER BY rowid")?;
        let ids = query
            .query_map([], |r| r.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        ids.iter().map(|id| self.service(id)).collect()
    }
    pub fn projects(&self) -> Result<Vec<Project>> {
        let mut query = self
            .connection
            .prepare("SELECT id,config FROM projects ORDER BY rowid")?;
        let rows = query
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        rows.into_iter()
            .map(|(id, json)| {
                Ok(Project {
                    id,
                    config: serde_json::from_str(&json)?,
                    state: "stopped".into(),
                })
            })
            .collect()
    }
    pub fn theme(&self) -> Result<String> {
        Ok(self
            .connection
            .query_row("SELECT value FROM preferences WHERE key='theme'", [], |r| {
                r.get(0)
            })
            .optional()?
            .unwrap_or_else(|| "dark".into()))
    }
    pub fn set_theme(&mut self, theme: &str) -> Result<()> {
        if !["dark", "light"].contains(&theme) {
            return Err(invalid("Theme must be dark or light."));
        }
        self.connection.execute("INSERT INTO preferences(key,value) VALUES('theme',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value",[theme])?;
        Ok(())
    }
    pub fn update_project(
        &mut self,
        id: &str,
        service_id: Option<String>,
        runtime: String,
    ) -> Result<()> {
        if let Some(service) = &service_id {
            self.service(service)?;
        }
        let mut project = self
            .projects()?
            .into_iter()
            .find(|p| p.id == id)
            .ok_or_else(|| invalid("Project no longer exists."))?;
        project.config.service_id = service_id;
        project.config.runtime = runtime;
        crate::validation::project(&mut project.config)?;
        self.connection.execute(
            "UPDATE projects SET config=?1 WHERE id=?2",
            params![serde_json::to_string(&project.config)?, id],
        )?;
        Ok(())
    }
}
