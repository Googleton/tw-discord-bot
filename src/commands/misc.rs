use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::Message;
use serenity::prelude::Context;

#[command]
async fn time_to(ctx: &Context, msg: &Message) -> CommandResult {
    
    Ok(())
}