use std::{
    error::Error,
    sync::Arc, time::Duration,
};
use chrono::Utc;
use log::{debug, error, info};
use lost_metrics_simulator::simulator::Simulator;
use tauri::{async_runtime::JoinHandle, App, AppHandle, Emitter, Listener, Manager};
use tokio::{runtime::{Handle, Runtime}, sync::Mutex, task};
use crate::{app_ready_state::AppReadyState, models::*, process_watcher::{self, ProcessWatcher}, processor::Processor, settings_manager::{self, SettingsManager}, updater::*};

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

pub fn setup_update_checker_callbacks(
    app_handle: AppHandle,
    app_updater: Arc<Mutex<AppUpdater>>,
    process_watcher: Arc<Mutex<ProcessWatcher>>) {
    {
        let app_updater = app_updater.clone();
        let app_handle_emit = app_handle.clone();
        app_handle.listen_any("check-update", move |event| {
            match app_updater.try_lock() {
                Ok(app_updater) => {

                    if app_updater.is_background_checker_running() {
                        app_updater.force_periodic_check();
                    }

                },
                Err(err) => {
                    app_handle_emit.emit("updater", "update-check-already-running").unwrap();
                },
            };
            
        });
    }

    {
        let app_updater = app_updater.clone();
        let process_watcher = process_watcher.clone();
        app_handle.listen_any("install-update", move |event| {
            let app_updater = app_updater.clone();
            let process_watcher = process_watcher.clone();
            tauri::async_runtime::spawn(async move {
                let mut app_updater = app_updater.lock().await;
                app_updater.stop().await;
                let mut process_watcher = process_watcher.lock().await;
                process_watcher.stop().unwrap();
            });
        });
    }
}

pub struct BackgroundWorker {
    app_handle: AppHandle,
    app_updater: Arc<Mutex<AppUpdater>>,
    process_watcher: Arc<Mutex<ProcessWatcher>>,
    app_ready_state: Arc<AppReadyState>,
    sniffer_settings: SnifferSettings,
    handle: Option<JoinHandle<anyhow::Result<()>>>
}

impl BackgroundWorker {
    pub fn new(
        app_handle: AppHandle,
        app_updater: Arc<Mutex<AppUpdater>>,
        process_watcher: Arc<Mutex<ProcessWatcher>>,
        app_ready_state: Arc<AppReadyState>,
        sniffer_settings: SnifferSettings) -> Self {
        Self {
            app_handle,
            app_updater,
            process_watcher,
            app_ready_state,
            sniffer_settings,
            handle: None
        }
    }

    pub fn start(&mut self) {
        let process_watcher = self.process_watcher.clone();
        let mut processor = Processor::new(self.app_handle.clone());
        let app_handle = self.app_handle.clone();
        let app_updater = self.app_updater.clone();
        let app_ready_state = self.app_ready_state.clone();
        let sniffer_settings = self.sniffer_settings.clone();

        let handle = tauri::async_runtime::spawn(async move {
            info!("waiting for load");
            app_ready_state.wait_for_ready();
            // setup_update_checker(app_handle.clone(), app_updater).await?;
    
            let rx = {
                let mut process_watcher = process_watcher.lock().await;
                process_watcher.start(&sniffer_settings.process_name, sniffer_settings.port)
            };
    
            let mut process_state = ProcessState::Unknown;
            let recv_timeout = Duration::from_secs(2);

            loop {
                let message = match rx.recv_timeout(recv_timeout) {
                    Ok(message) => {
                        process_state = message.clone();
                        message
                    },
                    Err(_) => {
                        let process_watcher = process_watcher.lock().await;
                        
                        if !process_watcher.is_running() {
                            break;
                        }
    
                        ProcessState::Unknown
                    },
                };

                let result = ProcessWatcherResult {
                    checked_on: Utc::now(),
                    state: process_state.clone()
                };

                app_handle.emit("process-watcher", result)?;
    
                match message {
                    ProcessState::ProcessListening(region) => {
                        processor.start(region);
                    },
                    ProcessState::ProcesStopped => {
                        processor.stop();
                    },
                    _ => {}
                }
            }
    
            anyhow::Ok(())
        });

        self.handle = Some(handle);
    }

    pub fn stop(&mut self) {

    }
}