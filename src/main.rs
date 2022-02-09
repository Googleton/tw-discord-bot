mod api;
mod commands;

use std::{collections::HashSet, env, sync::Arc};

use regex::Regex;

use commands::trib_interact::*;
use commands::tribal::*;
use serenity::framework::standard::macros::help;
use serenity::{
    async_trait,
    client::bridge::gateway::ShardManager,
    framework::standard::{
        macros::{group, hook},
        StandardFramework,
    },
    http::Http,
    model::{channel::Message, gateway::Ready},
    Client,
};

use serenity::framework::standard::{
    help_commands, Args, CommandGroup, CommandResult, HelpOptions,
};

use crate::api::models::TribalWars;
use crate::api::reports::generate_report_image;
use crate::api::tribalapi;
use serenity::model::prelude::*;
use serenity::prelude::{Context, EventHandler, TypeMapKey};
use tokio::sync::{Mutex, RwLock};
use tracing::{error, info};
use crate::tribalapi::download_api_data;

pub struct ShardManagerContainer;

pub struct TribalWarsState;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }

    async fn interaction_create(&self, _ctx: Context, _interaction: Interaction) {
        info!("Interaction created??");
    }
}

#[group]
#[commands(update, force_update, villages, tribe, tribe_members, travel, show_map)]
struct General;

#[help]
async fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to load .env file");

    tracing_subscriber::fmt::init();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let appid: u64 = env::var("APPLICATION_ID")
        .expect("Expected an application ID in the environment")
        .parse()
        .expect("Application ID invalid");

    let http = Http::new_with_token(&token);

    let (owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);
            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| c.owners(owners).prefix("_"))
        .normal_message(normal_message)
        .group(&GENERAL_GROUP)
        .help(&MY_HELP);

    let mut client = Client::builder(&token)
        .framework(framework)
        .event_handler(Handler)
        .application_id(appid)
        .await
        .expect("Err creating client");

    download_api_data(false).await;
    
    {
        let tw = TribalWars::load();
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
        data.insert::<TribalWarsState>(Arc::new(RwLock::new(tw)));
    }

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Could not register ctrl+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}

#[hook]
async fn normal_message(_ctx: &Context, msg: &Message) {
    if msg.author.bot {
        return;
    }

    // TODO: This can probably be improved
    let rgx = Regex::new(r"(\d{3})\|(\d{3})").unwrap();
    for capture in rgx.captures_iter(msg.content.as_str()) {
        tribalapi::download_api_data(false).await;
        let x = &capture[1].parse::<u32>().unwrap();
        let y = &capture[2].parse::<u32>().unwrap();
        send_village_embed(_ctx, msg, x, y).await;
        return;
    }

    // TODO: Probably move this to its own function to keep the code cleaner
    let msg_parts = msg.content.split(" ");
    for msg_part in msg_parts {
        if msg_part.contains("https://en125.tribalwars.net/public_report/") {
            match generate_report_image(msg_part) {
                Ok(_) => {}
                Err(error) => {
                    error!("Error generating report screenshot: {:?}", error);
                }
            };

            info!("Sending report image");

            msg.channel_id
                .send_message(&_ctx.http, |m| m.add_file("report.png"))
                .await;
        }
    }
}

impl TypeMapKey for TribalWarsState {
    type Value = Arc<RwLock<TribalWars>>;
}
