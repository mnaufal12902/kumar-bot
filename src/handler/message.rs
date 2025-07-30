use serenity::model::channel::Message;
use serenity::prelude::*;

use crate::commands;
use crate::token::registry::TokenRegistry;

pub async fn handle_message(ctx: Context, msg: Message) {
    if msg.author.bot {
        return;
    }

    let registry = TokenRegistry::new();
    let content = msg.content.to_lowercase();

    if content.starts_with("pb!ping") {
        commands::greeting::run(ctx.clone(), msg.clone()).await;
    } else if content.starts_with("pb!join") {
        commands::voice::join(ctx.clone(), msg.clone()).await;
    } else if content.starts_with("pb!leave") {
        commands::voice::leave(ctx.clone(), msg.clone()).await;
    } else if content.starts_with("pb!play") {
        commands::music::track::run(ctx.clone(), msg.clone(), registry).await;
    } else if content.starts_with("pb!pause") {
        commands::music::track::pause(ctx.clone(), msg.clone()).await;
    } else if content.starts_with("pb!resume") {
        commands::music::track::resume(ctx.clone(), msg.clone()).await;
    } else if content.starts_with("pb!skip") {
        commands::music::track::skip(ctx.clone(), msg.clone()).await;
    }
}
