use std::{sync::{Arc, Mutex}, thread::JoinHandle, time::Duration};

use chrono::Utc;
use log::debug;
use tauri::{AppHandle, Emitter};
use anyhow::*;
use tokio::{runtime::Runtime, time::sleep};
use uuid::Uuid;

use crate::{fake_encounter::FakeEncounter, models::{Boss, Encounter, Player}};

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

        let handle = std::thread::spawn(move || {
            let rt = Runtime::new().expect("Failed to create runtime");

            rt.block_on(async {
                let mut fake_encounter = FakeEncounter::new();

                loop {
    
                    fake_encounter.tick();
    
                    let encounter = fake_encounter.get();
        
                    app_handle.emit("encounter-update", encounter).unwrap();
        
                    sleep(duration).await;
                }
            })
        });
        self.handle = Some(handle);
    }

    pub async fn stop(&mut self) -> Result<()> {
        if let Some(handle) = self.handle.take() {
            handle.join()
                .map_err(|err| anyhow::anyhow!("{:?}", err))?;
        }

        Ok(())
    }
}