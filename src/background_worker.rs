use crate::{app_ready_state::AppReadyState, models::{ProcessState, ProcessWatcherResult, SnifferSettings}, process_watcher::ProcessWatcher, processor::Processor, updater::AppUpdater};
use std::{
    error::Error, sync::Arc, thread::JoinHandle, time::Duration
};
use chrono::Utc;
use log::{debug, error, info};
use lost_metrics_simulator::simulator::Simulator;
use tauri::{App, AppHandle, Emitter, Listener, Manager};
use tokio::{runtime::{Handle, Runtime}, sync::Mutex, task};
use anyhow::Result;

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

        let handle = std::thread::spawn(move || {
            let rt = Runtime::new().expect("Failed to create runtime");

            rt.block_on(async {
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
    
                    app_handle.emit("process-check", result)?;
        
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
            })
        });

        self.handle = Some(handle);
    }

    pub fn stop(&mut self) -> Result<()> {
        if let Some(handle) = self.handle.take() {
            handle.join()
                .map_err(|err| anyhow::anyhow!("{:?}", err))?;
        }

        Ok(())
    }
}