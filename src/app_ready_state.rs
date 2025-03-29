use std::sync::{Arc, Condvar, Mutex};

pub struct AppReadyState {
    ready: Mutex<bool>,
    condvar: Condvar,
}

impl AppReadyState {
    pub fn new() -> Self {
        Self {
            ready: Mutex::new(false),
            condvar: Condvar::new(),
        }
    }

    pub fn mark_ready(&self) {
        let mut is_ready = self.ready.lock().unwrap();
        *is_ready = true;
        self.condvar.notify_all();
    }

    pub fn wait_for_ready(&self) {
        let mut is_ready = self.ready.lock().unwrap();
        while !*is_ready {
            is_ready = self.condvar.wait(is_ready).unwrap();
        }
    }
}