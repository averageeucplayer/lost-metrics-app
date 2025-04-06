use std::{error::Error, sync::Arc, time::Duration};

use chrono::Utc;
use log::*;
use tauri::{async_runtime::JoinHandle, AppHandle, Emitter, Listener};
use tauri_plugin_updater::{Result, Update, Updater, UpdaterExt};
use tokio::{sync::{Mutex, Notify}, time::sleep};

use crate::{models::{UpdaterResult, UpdaterState}, process_watcher::ProcessWatcher};

pub struct AppUpdater {
    handle: Option<JoinHandle<()>>,
    downloaded: usize,
    has_downloaded: bool,
    updater: Option<Updater>,
    update_info: Option<Update>,
    wake_up_signal: Arc<Notify>,
}

impl AppUpdater {
    pub fn new(app_handle: AppHandle) -> Self {

        let handle = None;
        let downloaded = 0;
        let has_downloaded = false;
        let updater = app_handle.updater().ok();
        let update_info = None;
        let wake_up_signal = Arc::new(Notify::new());

        Self {
            handle,
            downloaded,
            has_downloaded,
            updater,
            update_info,
            wake_up_signal
        }
    }

    pub async fn check(&mut self) -> UpdaterState {

        if self.update_info.is_some() {
            return UpdaterState::NewVersion;
        }
       
        let updater = self.updater.as_ref().expect("Invalid state");

        let check_result = updater.check().await;

        match check_result {
            Ok(Some(update)) => {
                self.update_info = Some(update);
                UpdaterState::NewVersion
            },
            Ok(None) => UpdaterState::Unknown,
            // Edge case, either app is deployed for the first time or issues with github
            Err(err) => UpdaterState::Error(err.to_string()),
        }
    }

    pub(super) fn set_handle(&mut self, handle: JoinHandle<()>) {
        self.handle = Some(handle);
    }

    pub async fn download_and_install(&mut self) -> Result<()> {

        if self.has_downloaded {
            return Ok(());
        }

        let update_info = self.update_info.as_ref().expect("Invalid state");

        info!("would download and install");
        self.has_downloaded = true;
        // update_info.download_and_install(
        //     |chunk_length, _| {
        //         self.downloaded += chunk_length;
        //     },
        //     || {
        //         self.has_downloaded = true;
        //     },
        //     )
        //     .await?;

        Ok(())
    }

    pub fn is_background_checker_running(&self) -> bool {
        self.handle.is_some()
    }

    pub fn force_periodic_check(&self) {
        self.wake_up_signal.notify_one();
    }

    pub async fn stop(&mut self) -> Result<()> {
        if let Some(handle) = self.handle.take() {
            handle.await?;
        }

        Ok(())
    }
}

pub async fn run_periodically(app_updater: Arc<Mutex<AppUpdater>>, app_handle: AppHandle, duration: Duration) {

    let wake_up_signal = {
        let app_updater = app_updater.lock().await;
        app_updater.wake_up_signal.clone()
    };

    let handle = {
        let app_updater = app_updater.clone();
        tauri::async_runtime::spawn(async move {
            loop {
                tokio::select! {
                    _ = sleep(duration) => {},
                    _ = wake_up_signal.notified() => {},
                }

                let mut app_updater = app_updater.lock().await;
                let mut result = UpdaterResult {
                    checked_on: Utc::now(),
                    state: UpdaterState::Unknown,
                };

                app_handle.emit("updater", result).unwrap();
            }
        })
    };

    let mut app_updater = app_updater.lock().await;
    app_updater.set_handle(handle);
}

pub async fn setup_update_checker(app_handle: AppHandle, app_updater: Arc<Mutex<AppUpdater>>) -> anyhow::Result<()> {
    let update_check_timeout = Duration::from_secs(60);

    let update_result= {
        let mut app_updater = app_updater.lock().await;
        app_updater.check().await
    };

    match update_result {
        UpdaterState::NewVersion => {

            let mut result = UpdaterResult {
                checked_on: Utc::now(),
                state: UpdaterState::NewVersion,
            };
            app_handle.emit("updater", result)?;
            let mut app_updater = app_updater.lock().await;
            
            match app_updater.download_and_install().await {
                Ok(_) => {
                    info!("would restart")
                    // app_handle.restart();
                },
                Err(err) => {
                    error!("Could not download and install: {:?}", err);
                },
            }
        },
        UpdaterState::Error(error) => {
            error!("Update check: {:?}", error);
            run_periodically(app_updater, app_handle, update_check_timeout).await;
        },
        _ => {
            run_periodically(app_updater, app_handle, update_check_timeout).await;
        }
    }

    anyhow::Ok(())
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
                    let mut result = UpdaterResult {
                        checked_on: Utc::now(),
                        state: UpdaterState::Error(err.to_string()),
                    };

                    app_handle_emit.emit("updater", result).unwrap();
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
