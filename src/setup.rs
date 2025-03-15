use std::{error::Error, sync::{Arc, Mutex}};
use tauri::{App, Emitter, Listener, Manager};

pub fn setup_app(app: &mut App) -> Result<(), Box<dyn Error>> {
    Ok(())
}