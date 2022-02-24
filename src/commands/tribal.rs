use crate::api::models::{TribalWars, Tribe};
use crate::api::tribalapi::get_village_thumbnail;
use crate::{api, TribalWarsState};
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::Message;
use serenity::prelude::Context;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::Duration;

#[command]
async fn update(ctx: &Context, msg: &Message) -> CommandResult {
    let updated = api::tribalapi::update_api_data(false, ctx).await;

    if updated {
        msg.channel_id
            .say(&ctx.http, "Tribal Wars data updated")
            .await?;
    } else {
        msg.channel_id
            .say(
                &ctx.http,
                "Last update was less than an hour ago, try later",
            )
            .await?;
    }

    Ok(())
}

// Duplicates function above  -> Maybe serenity has a handler for that stuff? Or I can extract the inside to another function
#[command]
async fn force_update(ctx: &Context, msg: &Message) -> CommandResult {
    let updated = api::tribalapi::update_api_data(true, ctx).await;

    if updated {
        msg.channel_id
            .say(&ctx.http, "Tribal Wars data updated")
            .await?;
    } else {
        msg.channel_id
            .say(
                &ctx.http,
                "Last update was less than an hour ago, try later",
            )
            .await?;
    }

    Ok(())
}

#[command]
#[aliases("t")]
#[usage("<tribe name or tag>")]
async fn tribe(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    api::tribalapi::update_api_data(false, ctx).await;

    let tribe_name = args.rest();

    let tw = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<TribalWarsState>()
            .expect("Expected TribalWars in TypeMap.")
            .clone()
    };

    {
        let game = tw.write().await;
        let tribe = game.tribe_by_name(tribe_name);
        let players = game.players_by_tribe(tribe_name);

        if tribe.is_none() || players.is_none() {
            msg.channel_id
                .say(&ctx.http, "That tribe doesn't exist you fucking idiot")
                .await?;
            return Ok(());
        }

        let tribe = tribe.unwrap();

        msg.channel_id
            .send_message(&ctx.http, |m| {
                m.content("").embed(|e| {
                    e.title(format!(
                        "{} [{}] - {}p.",
                        tribe.name, tribe.tag, tribe.points
                    ))
                    .fields(vec![
                        ("Members", format!("{}", tribe.members).as_str(), true),
                        (
                            "Average",
                            format!("{}", tribe.all_points / tribe.members).as_str(),
                            true,
                        ),
                    ])
                    .url(format!(
                        "https://en125.tribalwars.net/game.php?screen=info_ally&id={}",
                        tribe.id
                    ))
                })
            })
            .await?;
    }

    Ok(())
}

#[command]
#[aliases("tm")]
#[usage("<tribe name or tag>")]
async fn tribe_members(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    api::tribalapi::update_api_data(false, ctx).await;

    let tribe_name = args.rest();

    let tw = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<TribalWarsState>()
            .expect("Expected TribalWars in TypeMap.")
            .clone()
    };

    {
        let game = tw.write().await;
        let tribe = game.tribe_by_name(tribe_name);
        let players = game.players_by_tribe(tribe_name);

        if tribe.is_none() || players.is_none() {
            msg.channel_id
                .say(&ctx.http, "That tribe doesn't exist you fucking idiot")
                .await?;
            return Ok(());
        }

        let tribe = tribe.unwrap();
        let mut players = players.unwrap();
        players.sort_by(|a, b| b.points.cmp(&a.points));

        let player_display: String = players
            .iter()
            .map(|p| format!("**{}**: {} villages, {}p.\n", p.name, p.villages, p.points))
            .collect();

        msg.channel_id
            .say(
                &ctx.http,
                format!("Players in {}:\n{}", tribe.name, player_display),
            )
            .await?;
    }

    Ok(())
}

#[command]
#[aliases("tt")]
async fn travel(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    // First off: Parse the two args
    let from = args.single::<String>()?;
    let to = args.single::<String>()?;

    let from_coords: Vec<&str> = from.split("|").collect();
    let from_x = from_coords[0].parse::<u32>().unwrap() as f64;
    let from_y = from_coords[1].parse::<u32>().unwrap() as f64;

    let to_coords: Vec<&str> = to.split("|").collect();
    let to_x = to_coords[0].parse::<u32>().unwrap() as f64;
    let to_y = to_coords[1].parse::<u32>().unwrap() as f64;

    // Then, convert to x,y coords

    // Compute distance in units
    let distance = ((to_x - from_x).powf(2.0) + (to_y - from_y).powf(2.0)).sqrt();

    let spear_duration = Duration::new(1080, 0);
    let sword_duration = Duration::new(1320, 0);
    let scout_duration = Duration::new(540, 0);
    let pala_duration = Duration::new(600, 0);
    let hcav_duration = Duration::new(660, 0);
    let ram_duration = Duration::new(1800, 0);
    let noble_duration = Duration::new(2100, 0);

    msg.channel_id.say(&ctx.http,
                       format!("Distance from {} to {} ({:.2} fields)\n\
                       **Spear/Axe/Archer**: {}\n**Sword**: {}\n**Scout**: {}\n**Pala/LCav/MArch**: {}\n**HCav**: {}\n**Ram/Cat**: {}\n**Knobbler**: {}",
                               format!("{}|{}", from_x, from_y),
                               format!("{}|{}", to_x, to_y),
                               distance,
                               duration_to_tw_display(&(spear_duration.mul_f64(distance))),
                               duration_to_tw_display(&(sword_duration.mul_f64(distance))),
                               duration_to_tw_display(&(scout_duration.mul_f64(distance))),
                               duration_to_tw_display(&(pala_duration.mul_f64(distance))),
                               duration_to_tw_display(&(hcav_duration.mul_f64(distance))),
                               duration_to_tw_display(&(ram_duration.mul_f64(distance))),
                               duration_to_tw_display(&(noble_duration.mul_f64(distance)))
                       )
        )
        .await?;
    Ok(())
}

#[command]
#[aliases("map")]
async fn show_map(_ctx: &Context, _msg: &Message) -> CommandResult {
    // todo: generate maps

    Ok(())
}

fn duration_to_tw_display(durr: &Duration) -> String {
    let hours = durr.as_secs() / 3600;
    let minutes = (durr.as_secs() - (hours * 3600)) / 60;
    let seconds = durr.as_secs() - (hours * 3600) - (minutes * 60);
    format!("{}:{:02}:{:02}", hours, minutes, seconds)
}

// fn tw_display_to_duration(tw_disp: String) -> Duration {
//     
// }

pub async fn send_village_embed(_ctx: &Context, msg: &Message, x: &u32, y: &u32) {
    let tw = {
        let data_read = _ctx.data.read().await;
        data_read
            .get::<TribalWarsState>()
            .expect("Expected TribalWars in TypeMap.")
            .clone()
    };

    {
        let game = tw.write().await;
        let village = game.village_at(x, y);
        if village.is_some() {
            let village = village.unwrap();

            let player = game.player(village.player_id);
            let tribe: Option<&Tribe>;

            if player.is_some() {
                tribe = game.tribe(player.unwrap().tribe_id);
            } else {
                tribe = None;
            }

            msg.channel_id
                .send_message(&_ctx.http, |m| {
                    m.content("").embed(|e| {
                        e.title(format!("{} ({}|{})", village.name, village.x, village.y))
                            .description(if player.is_some() {
                                let player = player.unwrap();
                                player.name.as_str()
                            } else {
                                "Barbarian"
                            })
                            .fields(vec![
                                ("Points", format!("{}p.", village.rank), true),
                                (
                                    "Tribe",
                                    if tribe.is_some() {
                                        let tribe = tribe.unwrap();
                                        tribe.name.clone()
                                    } else {
                                        "None".to_string()
                                    },
                                    true,
                                ),
                            ])
                            .url(format!(
                                "https://en125.tribalwars.net/game.php?screen=info_village&id={}",
                                village.id
                            ))
                            .thumbnail(get_village_thumbnail(village.rank, false))
                    })
                })
                .await;
        }
    }
}
