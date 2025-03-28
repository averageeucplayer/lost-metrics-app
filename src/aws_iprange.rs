use std::{fs::{self, File}, io::Write, path::PathBuf};

use log::info;
use reqwest::Client;
use anyhow::{Ok, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AwsIpRanges {
    #[serde(rename = "syncToken")]
    pub sync_token: String,
    #[serde(rename = "createDate")]
    pub create_date: String,
    pub prefixes: Vec<IpPrefix>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IpPrefix {
    pub ip_prefix: String,
    pub region: String,
    pub service: String,
    pub network_border_group: String,
}


pub struct AwsIpRange {
    url: String,
    cached_json: PathBuf,
    client: Client
}

impl AwsIpRange {

    pub fn new() -> Self {
        let url = "https://ip-ranges.amazonaws.com/ip-ranges.json".into();
        let cached_json = "ip-ranges.json".into();
        let client = Client::new();

        Self { url, cached_json, client }
    }

    pub async fn get(&self) -> Result<AwsIpRanges> {

        if self.cached_json.exists() {
            let file = File::open(&self.cached_json)?;
            let ranges: AwsIpRanges = serde_json::from_reader(file)?;

            return Ok(ranges);
        }

        let result = self.client.get(&self.url).send().await?;
        let json_str = result.text_with_charset("utf-8").await?;
    
        fs::write(&self.cached_json, &json_str)?;
        let ranges: AwsIpRanges = serde_json::from_str(&json_str)?;

        Ok(ranges)
    }
}

pub struct FakeIpRanges {

}

impl FakeIpRanges {
    pub fn new() -> Self {
        Self { }
    }

    pub async fn get(&self) -> Result<AwsIpRanges> {
        let data = AwsIpRanges {
            sync_token: "".into(),
            create_date: "".into(),
            prefixes: vec![
                IpPrefix {
                    ip_prefix: "127.0.0.0/8".to_string(),
                    network_border_group: "n/a".into(),
                    region: "EUC".into(),
                    service: "n/a".into()
                }
            ]
        };

        Ok(data)
    }
}