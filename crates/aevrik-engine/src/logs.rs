mod terminal;

use crate::model::LogEntry;
use std::{
    collections::VecDeque,
    io::Read,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Default)]
pub(crate) struct Logs(Arc<Mutex<VecDeque<LogEntry>>>);
impl Logs {
    pub fn push(&self, service: &str, level: &str, message: impl Into<String>) {
        if let Ok(mut logs) = self.0.lock() {
            if logs.len() >= 800 {
                logs.pop_front();
            }
            logs.push_back(LogEntry {
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
                service: service.into(),
                level: level.into(),
                message: message.into(),
            });
        }
    }
    pub fn snapshot(&self) -> Vec<LogEntry> {
        self.0
            .lock()
            .map(|logs| logs.iter().cloned().collect())
            .unwrap_or_default()
    }
    pub fn clear(&self) {
        if let Ok(mut logs) = self.0.lock() {
            logs.clear();
        }
    }
    pub fn capture(
        &self,
        reader: impl Read + Send + 'static,
        service: String,
        level: &'static str,
    ) {
        let logs = self.clone();
        std::thread::spawn(move || logs.read_output(reader, &service, level));
    }
    fn push_output(&self, service: &str, level: &str, line: &[u8]) {
        let text = String::from_utf8_lossy(line);
        let message = terminal::message(text.trim_end_matches('\r'));
        if !message.trim().is_empty() {
            self.push(service, terminal::level(message, level), message);
        }
    }

    fn read_output(&self, mut reader: impl Read, service: &str, level: &str) {
        let mut terminal = terminal::Terminal::default();
        let mut buffer = [0u8; 2048];
        let mut line = Vec::new();
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    for &byte in &buffer[..n] {
                        if !terminal.text_byte(byte) {
                            continue;
                        }
                        // Flush before a new UTF-8 codepoint, never inside one.
                        if byte == b'\n'
                            || line.len() >= 8196
                            || (line.len() >= 8192 && byte & 0xc0 != 0x80)
                        {
                            self.push_output(service, level, &line);
                            line.clear();
                        }
                        if byte != b'\n' {
                            line.push(byte);
                        }
                    }
                }
                Err(error) => {
                    self.push_output(service, level, &line);
                    line.clear();
                    self.push(service, "ERROR", format!("Log capture: {error}"));
                    break;
                }
            }
        }
        self.push_output(service, level, &line);
    }
}

#[cfg(test)]
mod tests;
