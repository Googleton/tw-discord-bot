use std::sync::Arc;
use std::time::Duration;
use serenity::framework::standard::{Args, CommandResult};
use serenity::framework::standard::macros::command;
use serenity::{
    async_trait,
    builder::{CreateActionRow, CreateButton, CreateSelectMenu, CreateSelectMenuOption},
    client::{Context, EventHandler},
    futures::StreamExt,
    model::{
        channel::{Message, ReactionType},
        interactions::{
            message_component::ButtonStyle,
            InteractionApplicationCommandCallbackDataFlags,
            InteractionResponseType,
        },
    },
    Client,
};
use tokio::sync::RwLock;
use crate::{api, TribalWarsState};
use crate::api::models::{TribalWars, Village};
use tracing::{info, debug};
use crate::api::tribalapi;
use crate::commands::tribal::send_village_embed;

struct PlayerInteract;

impl PlayerInteract {
    fn select_option(vill: &Village) -> CreateSelectMenuOption {
        let mut opt = CreateSelectMenuOption::default();
        // This is what will be shown to the user
        opt.label(format!("{} - {}|{} - {}p.", vill.name, vill.x, vill.y, vill.rank));
        // This is used to identify the selected value
        opt.value(format!("{}.{}", vill.x, vill.y));
        opt
    }

    fn select_menu(vills: Vec<&Village>) -> CreateSelectMenu {
        let mut menu = CreateSelectMenu::default();
        menu.custom_id("village_show_id");
        menu.placeholder("Pick a village");

        menu.options(|f| {
            for vill in vills {
                f.add_option(PlayerInteract::select_option(vill));
            }
            f
        });

        menu
    }

    fn action_row(vills: Vec<&Village>) -> CreateActionRow {
        let mut ar = CreateActionRow::default();

        ar.add_select_menu(PlayerInteract::select_menu(vills));
        ar
    }
}


#[command]
#[aliases("v")]
async fn villages(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    tribalapi::update_api_data(false, ctx).await;

    let player_name = args.rest();

    let tw = {
        let data_read = ctx.data.read().await;
        data_read.get::<TribalWarsState>().expect("Expected TribalWars in TypeMap.").clone()
    };


    let m = {
        let game = tw.write().await;
        let villages = game.villages_for(player_name);

        if villages.is_none() {
            return Ok(());
        }

        let vills = villages.unwrap();

        msg
        .channel_id
        .send_message(&ctx, |m| {
            m.content(format!("Villages for {}", player_name));
            m.components(|c| c.add_action_row(PlayerInteract::action_row(vills)));
            m
        })
        .await
        .unwrap()
    };

    let mci =
        match m.await_component_interaction(&ctx).timeout(Duration::from_secs(60 * 3)).await {
            Some(ci) => ci,
            None => {
                m.reply(&ctx, "Timed out").await.unwrap();
                return Ok(());
            },
        };

    mci.create_interaction_response(&ctx, |r| {
        r.kind(InteractionResponseType::UpdateMessage);
        r.interaction_response_data(|d| {
            d
        })
    })
        .await
        .unwrap();

    let village_id = mci.data.values.get(0).unwrap().as_str();
    let village_coords: Vec<&str> = village_id.split(".").collect();
    let x = village_coords[0].parse::<u32>().unwrap();
    let y = village_coords[1].parse::<u32>().unwrap();

    send_village_embed(ctx, msg, &x, &y).await;

    m.delete(&ctx).await.unwrap();

    Ok(())
}