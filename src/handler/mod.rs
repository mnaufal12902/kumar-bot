mod message;

use message::handle_message;
use serenity::all::{
    ActivityData, ActivityType, Command, CreateCommand, CreateInteractionResponse, CreateInteractionResponseMessage, Guild, Interaction, OnlineStatus
};
use serenity::async_trait;
use serenity::model::{channel::Message, gateway::Ready};
use serenity::prelude::*;

use crate::utils::serenity_utils;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        handle_message(ctx, msg).await;
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            if command.data.name == "ready" {
                let _ = command.create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new().content("Bot is ready"),
                    ),
                ).await;
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        ctx.set_presence(
            Some(ActivityData {
                name: "on Sleeping".to_string(),
                kind: ActivityType::Playing,
                state: None,
                url: None,
            }),
            OnlineStatus::Idle,
        );

        tracing::info!("{} is online!", ready.user.name);

        let builder = CreateCommand::new("ready").description("Check if bot is ready");
        let _ = Command::create_global_command(&ctx.http, builder).await;
    }

    async fn guild_create(&self, ctx: Context, guild: Guild, is_new: Option<bool>) {
        if let Some(true) = is_new {
            let _ = serenity_utils::create_channel_from_guild(ctx, guild).await;
        }
    }
}
