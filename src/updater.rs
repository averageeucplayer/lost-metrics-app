use std::{error::Error, sync::Arc, time::Duration};

use log::*;
use tauri::{async_runtime::JoinHandle, AppHandle, Emitter};
use tauri_plugin_updater::{Result, Update, Updater, UpdaterExt};
use tokio::{sync::{Mutex, Notify}, time::sleep};

pub enum UpdateResult {
    Unknown,
    LatestVersion,
    NewVersion,
    Error(Box<dyn Error + Send + Sync>)
}

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

    pub async fn check(&mut self) -> UpdateResult {

        if self.update_info.is_some() {
            return UpdateResult::NewVersion;
        }
       
        let updater = self.updater.as_ref().expect("Invalid state");

        let check_result = updater.check().await;

        match check_result {
            Ok(Some(update)) => {
                self.update_info = Some(update);
                UpdateResult::NewVersion
            },
            Ok(None) => UpdateResult::Unknown,
            Err(err) => UpdateResult::Error(err.into()),
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

        update_info.download_and_install(
            |chunk_length, _| {
                self.downloaded += chunk_length;
            },
            || {
                self.has_downloaded = true;
            },
            )
            .await?;

        Ok(())
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
                
                match app_updater.check().await {
                    UpdateResult::Unknown => {
                        app_handle.emit("app-state", "unknown").unwrap();
                    },
                    UpdateResult::NewVersion => {
                        app_handle.emit("app-state", "new-version").unwrap();
                        return;
                    },
                    UpdateResult::LatestVersion => {
                        app_handle.emit("app-state", "latest-version").unwrap();
                    },
                    UpdateResult::Error(error) => {
                        error!("Periodic update check: {:?}", error);
                        app_handle.emit("app-state", "latest-version").unwrap();
                    }
                }
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
        UpdateResult::NewVersion => {

            app_handle.emit("app-state", "new-version")?;
            let mut app_updater = app_updater.lock().await;
            
            match app_updater.download_and_install().await {
                Ok(_) => {
                    app_handle.restart();
                },
                Err(err) => {
                    error!("Could not download and install: {:?}", err);
                },
            }
        },
        UpdateResult::Error(error) => {
            error!("Update check: {:?}", error);
            run_periodically(app_updater, app_handle, update_check_timeout).await;
        },
        _ => {
            run_periodically(app_updater, app_handle, update_check_timeout).await;
        }
    }

    anyhow::Ok(())
}