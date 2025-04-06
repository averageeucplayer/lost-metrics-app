use std::{
    error::Error,
    sync::Arc, time::Duration,
};
use chrono::Utc;
use log::{debug, error, info};
use lost_metrics_simulator::simulator::Simulator;
use tauri::{async_runtime::JoinHandle, App, AppHandle, Emitter, Listener, Manager};
use tokio::{runtime::{Handle, Runtime}, sync::Mutex, task};
use crate::{app_ready_state::AppReadyState, background_worker::BackgroundWorker, models::*, process_watcher::{self, ProcessWatcher}, processor::Processor, settings_manager::{self, SettingsManager}, updater::*};

pub fn setup_app(app: &mut App) -> Result<(), Box<dyn Error>> {
    #[cfg(debug_assertions)]
    {
        let window = app.get_webview_window("main").unwrap();
        window.open_devtools();
    }

    let rt = Handle::current();
    let mut settings_manager = SettingsManager::new("settings.json".into());

    let settings = task::block_in_place(|| {
        rt.block_on(async { settings_manager.get_or_create_default().await })
    })?;

    let settings_manager = Arc::new(Mutex::new(settings_manager));

    let app_handle = app.handle().clone();
    let version = app_handle.package_info().version.clone();
    let app_updater = AppUpdater::new(app_handle.clone());
    let app_updater: Arc<Mutex<AppUpdater>> = Arc::new(Mutex::new(app_updater));
    let process_watcher: Arc<Mutex<ProcessWatcher>> = Arc::new(Mutex::new(ProcessWatcher::new(settings.sniffer.check_interval)));
   
    let simulator = Arc::new(Simulator::new());
    let app_ready_state: Arc<AppReadyState> = Arc::new(AppReadyState::new());
        
    app.manage(simulator.clone());
    app.manage(settings_manager.clone());
    app.manage(app_ready_state.clone());

    setup_update_checker_callbacks(
        app_handle.clone(),
        app_updater.clone(),
        process_watcher.clone());

    let mut background_worker = BackgroundWorker::new(
        app_handle,
        app_updater,
        process_watcher,
        app_ready_state,
        settings.sniffer
    );
    background_worker.start();
 

    Ok(())
}