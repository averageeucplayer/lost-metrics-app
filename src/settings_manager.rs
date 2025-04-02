use std::{error::Error, path::PathBuf};

use tokio::fs::File;

use crate::models::Settings;


pub struct SettingsManager {
    path: PathBuf,
    settings: Option<Settings>
}

impl SettingsManager {
    pub fn new(path: PathBuf) -> Self {
        Self { 
            path,
            settings: None
        }
    }

    pub async fn save(&mut self, settings: &Settings) -> Result<(), Box<dyn Error>> {
        let file = File::create(&self.path).await?;
        serde_json::to_writer_pretty(file.into_std().await, &settings)?;

        Ok(())
    }

    pub async fn get_or_create_default(&mut self) -> Result<Settings, Box<dyn Error>> {

        if let Some(settings) = self.settings.clone() {
            return Ok(settings);
        }
        
        if self.path.exists() {
            let file = File::open(&self.path).await?;
            let settings = serde_json::from_reader(file.into_std().await)?;
            return Ok(settings);
        }

        let bytes = include_bytes!("../default_settings.json");
        let settings = serde_json::from_slice(bytes)?;
        
        let file = File::create(&self.path).await?;
        serde_json::to_writer_pretty(file.into_std().await, &settings)?;

        Ok(settings)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::settings_manager;

    use super::*;

    #[tokio::test]
    async fn should_create_settings() {

        let mut settings = Settings::default();
        settings.sniffer.check_interval = Duration::from_secs(10);

        let test = serde_json::to_string_pretty(&settings).unwrap();

        println!("{test}");
        
        let mut settings_manager = SettingsManager::new("settings.json".into());

        let result = settings_manager.get_or_create_default().await.unwrap();
    }
}