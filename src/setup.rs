use std::{
    error::Error,
    sync::Arc, time::Duration,
};
use log::{debug, error, info};
use tauri::{async_runtime::JoinHandle, App, AppHandle, Emitter, Listener, Manager};
use tokio::{runtime::{Handle, Runtime}, sync::Mutex};
use crate::{models::Message, process_watcher::{self, ProcessWatcher}, processor::Processor, updater::*};

pub fn setup_app(app: &mut App) -> Result<(), Box<dyn Error>> {
    // #[cfg(debug_assertions)]
    // {
    //     let window = app.get_webview_window("main").unwrap();
    //     window.open_devtools();
    // }

    let app_handle = app.handle().clone();
    let version = app_handle.package_info().version.clone();
    let app_updater = AppUpdater::new(app_handle.clone());
    let app_updater: Arc<Mutex<AppUpdater>> = Arc::new(Mutex::new(app_updater));
    let process_watcher: Arc<Mutex<ProcessWatcher>> = Arc::new(Mutex::new(ProcessWatcher::new()));

    {
        let app_updater = app_updater.clone();
        let app_handle_emit = app_handle.clone();
        app_handle.listen_any("check-update", move |event| {
            match app_updater.try_lock() {
                Ok(app_updater) => {
                    app_updater.force_periodic_check();
                },
                Err(err) => {
                    app_handle_emit.emit("app-state", "update-check-already-running").unwrap();
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

    let mut background_worker = BackgroundWorker::new(app_handle, app_updater, process_watcher);
    background_worker.start();

    Ok(())
}

pub struct BackgroundWorker {
    app_handle: AppHandle,
    app_updater: Arc<Mutex<AppUpdater>>,
    process_watcher: Arc<Mutex<ProcessWatcher>>,
    handle: Option<JoinHandle<anyhow::Result<()>>>
}

impl BackgroundWorker {
    pub fn new(
        app_handle: AppHandle,
        app_updater: Arc<Mutex<AppUpdater>>,
        process_watcher: Arc<Mutex<ProcessWatcher>>,) -> Self {
        Self {
            app_handle,
            app_updater,
            process_watcher,
            handle: None
        }
    }

    pub fn start(&mut self) {
        let process_watcher = self.process_watcher.clone();
        let mut processor = Processor::new(self.app_handle.clone());
        let app_handle = self.app_handle.clone();
        let app_updater = self.app_updater.clone();

        let handle = tauri::async_runtime::spawn(async move {
            setup_update_checker(app_handle.clone(), app_updater).await?;
    
            let process_name = "client_server.exe";
            let port = 6041;
            let timeout = Duration::from_secs(2);
    
            let rx = {
                let mut process_watcher = process_watcher.lock().await;
                process_watcher.start(process_name, port)
            };
    
            loop {
                let message = match rx.recv_timeout(timeout) {
                    Ok(message) => message,
                    Err(_) => {
                        let process_watcher = process_watcher.lock().await;
                        
                        if !process_watcher.is_running() {
                            break;
                        }
    
                        Message::Unknown
                    },
                };
    
                match message {
                    Message::ProcessListening(region) => {
                        app_handle.emit("app-state", "process-listening")?;
                        processor.start(region);
                    },
                    Message::ProcesStopped => {
                        app_handle.emit("app-state", "process-stopped")?;
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