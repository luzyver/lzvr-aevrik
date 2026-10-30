use crate::{
    error::{Result, invalid},
    job::{Job, prepare},
    logs::Logs,
    model::Service,
};
use std::{
    collections::HashMap,
    net::TcpListener,
    process::{Child, Command, Stdio},
};

struct Managed {
    child: Child,
    job: Option<Job>,
}
pub(crate) struct Processes {
    active: HashMap<String, Managed>,
    exited: HashMap<String, Option<i32>>,
    pub logs: Logs,
}
impl Processes {
    pub fn new() -> Self {
        Self {
            active: HashMap::new(),
            exited: HashMap::new(),
            logs: Logs::default(),
        }
    }
    pub fn poll(&mut self) -> Result<()> {
        let mut finished = Vec::new();
        for (id, process) in &mut self.active {
            if let Some(status) = process.child.try_wait()? {
                finished.push((id.clone(), status.code()));
            }
        }
        for (id, code) in finished {
            self.active.remove(&id);
            self.exited.insert(id.clone(), code);
            self.logs
                .push(&id, "INFO", format!("Process exited with code {code:?}."));
        }
        Ok(())
    }
    pub fn decorate(&self, service: &mut Service) {
        if let Some(process) = self.active.get(&service.id) {
            service.state = "running".into();
            service.pid = Some(process.child.id());
        } else if let Some(code) = self.exited.get(&service.id) {
            service.state = "exited".into();
            service.exit_code = *code;
        }
    }
    pub fn start(&mut self, service: &Service) -> Result<()> {
        self.poll()?;
        if self.active.contains_key(&service.id) {
            return Err(invalid("This service is already running."));
        }
        let mut config = service.config.clone();
        crate::validation::service(&mut config)?;
        if let Some(port) = config.port {
            let listener = TcpListener::bind(("127.0.0.1", port)).map_err(|_| {
                invalid(format!(
                    "Port {port} is unavailable. Existing processes were left running."
                ))
            })?;
            drop(listener);
        }
        let mut command = Command::new(&config.executable);
        command
            .args(&config.args)
            .current_dir(&config.cwd)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        prepare(&mut command);
        let mut child = command.spawn()?;
        let job = match Job::attach(&child) {
            Ok(job) => job,
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(error.into());
            }
        };
        if let Some(stdout) = child.stdout.take() {
            self.logs.capture(stdout, service.id.clone(), "INFO");
        }
        if let Some(stderr) = child.stderr.take() {
            self.logs.capture(stderr, service.id.clone(), "ERROR");
        }
        self.logs.push(
            &service.id,
            "INFO",
            format!("Started {} (PID {}).", config.name, child.id()),
        );
        self.active.insert(
            service.id.clone(),
            Managed {
                child,
                job: Some(job),
            },
        );
        self.exited.remove(&service.id);
        Ok(())
    }
    pub fn stop(&mut self, id: &str) -> Result<()> {
        if let Some(mut process) = self.active.remove(id) {
            // Closing the Job kills descendants as well as the root before waiting.
            drop(process.job.take());
            #[cfg(not(windows))]
            process.child.kill()?;
            process.child.wait()?;
            self.logs.push(id, "INFO", "Stopped owned process tree.");
        }
        self.exited.remove(id);
        Ok(())
    }
    pub fn shutdown(&mut self) -> Result<()> {
        let ids: Vec<_> = self.active.keys().cloned().collect();
        let mut first = None;
        for id in ids {
            if let Err(error) = self.stop(&id) {
                first.get_or_insert(error);
            }
        }
        match first {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }
}
impl Drop for Processes {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}
