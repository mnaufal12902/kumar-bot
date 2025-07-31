use serenity::all::{CreateEmbed, CreateMessage};
use serenity::model::channel::Message;
use serenity::prelude::*;

use crate::commands;
use crate::token::registry::TokenRegistry;
use crate::utils::serenity_utils;

pub async fn handle_message(ctx: Context, msg: Message) {
    if msg.author.bot {
        return;
    }

    let channel_name = r"kumar-channel";
    let guild_id = match msg.guild_id {
        Some(gid) => gid,
        None => {
            return;
        }
    };

    let registry = TokenRegistry::new();
    let content = msg.content.to_lowercase();

    let result = ctx.http().get_channels(guild_id).await;

    let channels = match result {
        Ok(channels) => channels,
        Err(err) => {
            tracing::error!("Error: {:?}", err);
            return;
        }
    };

    let allowed_channel = match channels.iter().find(|f| f.name == channel_name) {
        Some(c) => c.clone(),
        None => {
            let result = serenity_utils::create_channel_from_id(&ctx, &msg).await;

            match result {
                Ok(r) => r,
                Err(err) => {
                    tracing::error!("Error : {:?}", err);
                    return;
                }
            }
        }
    };

    if msg.channel_id != allowed_channel.id {
        let embed = CreateEmbed::default()
            .description("Messages must be sent in this channel")
            .color(0xFF0000);

        let builder = CreateMessage::default().embed(embed);
        let _ = allowed_channel.id.send_message(&ctx.http, builder).await;
        return
    }

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
    } else if content.starts_with("pb!volume") {
        commands::music::track::volume(ctx.clone(), msg.clone()).await;
    }
}
