use std::time::Duration;

use chrono::Utc;
use log::debug;
use tauri::{async_runtime::JoinHandle, AppHandle, Emitter};
use anyhow::*;
use tokio::time::sleep;
use uuid::Uuid;

use crate::models::Encounter;

pub struct Processor {
    app_handle: AppHandle,
    handle: Option<JoinHandle<()>>
}

impl Processor {
    pub fn new(app_handle: AppHandle) -> Self {
        Self { 
            app_handle,
            handle: None
        }
    }

    pub fn start(&mut self, region: String) {

        debug!("start");
        let app_handle = self.app_handle.clone();
        let duration  = Duration::from_secs(1);

        // TO-DO Download dll from https://github.com/averageeucplayer/lost-metrics-sniffer/releases/latest
        // Then follow logic as in lost-metrics-console
        let handle: JoinHandle<()> = tauri::async_runtime::spawn(async move {
            let mut encounter = Encounter {
                id: Uuid::now_v7(),
                updated_on: Utc::now(),
                total_damage: 0,
            };

            loop {
                
                encounter.total_damage += 1;
    
                app_handle.emit("encounter-update", &encounter).unwrap();
    
                sleep(duration).await;
            }
        });

        self.handle = Some(handle);
    }

    pub async fn stop(&mut self) -> Result<()> {
        if let Some(handle) = self.handle.take() {
            handle.await?;
        }

        Ok(())
    }
}