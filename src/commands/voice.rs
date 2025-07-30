use serenity::{
    all::{ActivityData, ActivityType, OnlineStatus, User},
    client::Context,
    model::prelude::Message,
};
use songbird::Event;

use crate::{
    bot::MusicStateKey,
    commands::music::{event::OnDisconnect, queue::GuildMusicSession},
    utils::serenity_utils,
};

pub async fn join(ctx: Context, msg: Message) {
    if let Some(guild_id) = msg.guild_id {
        let user_id = msg.author.id;

        let user_info = user_id.to_user(&ctx.http).await;

        let user: Option<User> = match user_info {
            Ok(user) => Some(user),
            Err(_) => None,
        };

        let maybe_channel_id = ctx.cache.guild(guild_id).and_then(|guild| {
            guild
                .voice_states
                .get(&user_id)
                .and_then(|voice| voice.channel_id)
        });

        if let Some(channel_id) = maybe_channel_id {
            if let Some(manager) = songbird::get(&ctx).await {
                let data = ctx.data.read().await;
                let music_state = data.get::<MusicStateKey>().unwrap();
                let mut state = music_state.lock().await;

                if state.music_sessions.get(&guild_id).is_none() {
                    match manager.join(guild_id, channel_id).await {
                        Ok(join_result) => {
                            ctx.set_presence(Some(ActivityData {
                                name: format!(
                                    "{}",
                                    user.map(|u| u.name).unwrap_or("someone".to_string())
                                ),
                                kind: ActivityType::Listening,
                                url: None,
                                state: None,
                            }), OnlineStatus::Online);

                            let sessions =
                                state.music_sessions.entry(guild_id).or_insert_with(|| {
                                    GuildMusicSession::new(
                                        Some(join_result.clone()),
                                        guild_id.clone(),
                                        channel_id.clone(),
                                    )
                                });

                            if let Some(call) = &sessions.voice_state.call {
                                let mut call_guard = call.lock().await;

                                let _ = call_guard.add_global_event(
                                    Event::Core(songbird::CoreEvent::DriverDisconnect),
                                    OnDisconnect {
                                        ctx: ctx.clone(),
                                        msg: msg.clone(),
                                    },
                                );
                            }
                        }
                        Err(_err) => {
                            serenity_utils::send_embed(
                                &ctx,
                                &msg,
                                "Failed to connect voice channel 😞",
                                0xFF0000,
                            )
                            .await;
                        }
                    };
                }
            };
        } else {
            serenity_utils::send_embed(&ctx, &msg, "You're not in voice channel", 0xFF0000).await;
        }
    };
}

pub async fn leave(ctx: Context, msg: Message) {
    ctx.set_presence(
        Some(ActivityData {
            name: "on Sleeping".to_string(),
            kind: ActivityType::Playing,
            state: None,
            url: None,
        }),
        OnlineStatus::Idle,
    );

    let guild_id = match msg.guild_id {
        Some(g) => g,
        None => {
            serenity_utils::send_embed(&ctx, &msg, "Failed to get guild id 😞", 0xFF0000).await;
            return;
        }
    };

    let manager = match songbird::get(&ctx).await {
        Some(m) => m,
        None => return,
    };

    let data = ctx.data.read().await;
    let music_state = data.get::<MusicStateKey>().unwrap();
    let mut state = music_state.lock().await;

    let _ = manager.remove(guild_id).await;

    state.music_sessions.remove(&guild_id);
}
