//! Coordinate mutations with the final installer handoff.
use std::sync::{RwLock, RwLockReadGuard};

#[derive(Default)]
pub struct Operations(pub RwLock<bool>);

impl Operations {
    pub fn enter(&self) -> Result<RwLockReadGuard<'_, bool>, String> {
        let guard = self
            .0
            .try_read()
            .map_err(|_| "Aevrik is preparing to update. Try again after it restarts.")?;
        if *guard {
            return Err("Aevrik is restarting for an update.".into());
        }
        Ok(guard)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_operation_prevents_handoff_and_closing_rejects_new_operations() {
        let operations = Operations::default();
        let active = operations.enter().unwrap();
        assert!(operations.0.try_write().is_err());
        drop(active);
        let mut handoff = operations.0.try_write().unwrap();
        assert!(operations.enter().is_err());
        *handoff = true;
        drop(handoff);
        assert!(operations.enter().is_err());
    }

    #[test]
    fn failed_handoff_leaves_operations_available() {
        let operations = Operations::default();
        drop(operations.0.try_write().unwrap());
        assert!(operations.enter().is_ok());
    }
}
