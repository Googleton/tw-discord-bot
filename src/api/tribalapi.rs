use csv::WriterBuilder;
use serde::{Deserialize, Serialize};
use serenity::prelude::Context;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::time::SystemTime;
use serenity::model::gateway::{Activity, ActivityType};
use serenity::model::prelude::OnlineStatus;
use tokio::sync::RwLock;

use crate::api::models::TribalWars;
use crate::{TribalWarsState};
use tracing::{info, debug};

#[derive(Serialize, Deserialize)]
struct LastUpdate {
    when: SystemTime,
}

pub async fn update_api_data(bypass: bool, ctx: &Context) -> bool {
    ctx.set_presence(Some(Activity::playing("updating TW data")), OnlineStatus::DoNotDisturb).await;
    let updated = download_api_data(bypass).await;

    if updated {
        {
            let tw = TribalWars::load();
            let mut data = ctx.data.write().await;
            data.insert::<TribalWarsState>(Arc::new(RwLock::new(tw)));
        }
    }

    ctx.reset_presence().await;
    return updated;
}

pub async fn download_api_data(bypass: bool) -> bool {
    let mut should_update = false;

    info!("Checking update for TW API data");
    match check_last_update() {
        Ok(res) => {
            should_update = res;
        }
        Err(err) => {
            eprintln!("Couldn't read update file {:?}", err)
        }
    }

    if should_update || bypass {
        info!("Updating TW API data");
        download_file("https://en125.tribalwars.net/map/village.txt").await;
        download_file("https://en125.tribalwars.net/map/ally.txt").await;
        download_file("https://en125.tribalwars.net/map/player.txt").await;
        match write_last_update() {
            Ok(_) => {}
            Err(err) => {
                eprintln!("Couldn't write update file {:?}", err)
            }
        }

        true
    } else {
        info!("Last update was less than an hour ago, no need to download new data");
        false
    }
}

async fn download_file(target: &str) -> Result<(), Box<dyn Error>> {
    debug!("Downloading file {}", target);
    let response = reqwest::get(target).await?;
    let fname = response
        .url()
        .path_segments()
        .and_then(|segments| segments.last())
        .and_then(|name| if name.is_empty() { None } else { Some(name) })
        .unwrap_or("tmp.bin");

    let dest = File::create(fname);
    let content = response.text().await?;

    match dest {
        Ok(mut file) => file.write(content.as_bytes()),
        Err(e) => panic!("Uh oh, stinky. {:?}", e),
    };

    Ok(())
}

fn check_last_update() -> Result<bool, Box<dyn Error>> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(Path::new("update.csv"))?;
    for result in rdr.deserialize() {
        let update: LastUpdate = result?;

        match SystemTime::now().duration_since(update.when) {
            Ok(n) => {
                return Ok(n.as_secs() > 3600);
            }
            Err(_) => eprintln!("Couldn't read last update"),
        }
    }

    return Ok(true);
}

fn write_last_update() -> Result<(), Box<dyn Error>> {
    fs::remove_file("update.csv")?;

    let mut wtr = WriterBuilder::new()
        .has_headers(false)
        .from_path("update.csv")?;
    wtr.serialize(LastUpdate {
        when: SystemTime::now(),
    })?;

    Ok(())
}

pub fn get_village_thumbnail(village_points: u32, _barb: bool) -> &'static str {
    match village_points {
        0..=299 => "https://dsen.innogamescdn.com/asset/6f4647d7/graphic///map_new/v1.png",
        300..=999 => "https://dsen.innogamescdn.com/asset/6f4647d7/graphic///map_new/v2.png",
        1000..=2999 => "https://dsen.innogamescdn.com/asset/6f4647d7/graphic///map_new/v3.png",
        3000..=8999 => "https://dsen.innogamescdn.com/asset/6f4647d7/graphic///map_new/v4.png",
        9000..=10999 => "https://dsen.innogamescdn.com/asset/6f4647d7/graphic///map_new/v5.png",
        11000..=99999 => "https://dsen.innogamescdn.com/asset/6f4647d7/graphic///map_new/v6.png",
        _ => "https://dsen.innogamescdn.com/asset/6f4647d7/graphic///map_new/v1.png",
    }
}
