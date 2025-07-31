use std::sync::{Arc, Weak};

use serenity::async_trait;
use songbird::TrackEvent;
use songbird::events::{Event, EventContext, EventHandler};
use tokio::sync::Mutex;

use crate::commands;
use crate::commands::music::queue::BotMusicState;
use crate::utils::serenity_utils;

pub struct OnEnd {
    pub ctx: serenity::all::Context,
    pub msg: serenity::all::Message,
    pub guild_id: serenity::all::GuildId,
    pub call: Weak<Mutex<songbird::Call>>,
    pub shared_state: Weak<Mutex<BotMusicState>>,
}

pub struct OnPlayable {
    pub ctx: serenity::all::Context,
    pub msg: serenity::all::Message,
    pub guild_id: serenity::all::GuildId,
    pub shared_state: Weak<Mutex<BotMusicState>>,
}

pub struct OnDisconnect {
    pub ctx: serenity::all::Context,
    pub msg: serenity::all::Message,
}

#[async_trait]
impl EventHandler for OnEnd {
    async fn act(&self, _e_ctx: &EventContext<'_>) -> Option<Event> {
        let shared_arc = self.shared_state.upgrade()?;
        let mut shared = shared_arc.lock().await;
        let guild = shared.music_sessions.get_mut(&self.guild_id)?;

        let channel = &mut guild.voice_state;

        if let Some((_, metadata)) = &channel.now_playing {
            if let Some(meta) = metadata {
                tracing::info!("🎵 Finished playing {:?} {:?}", meta.title, meta.artist);
            };

            channel.now_playing = None;
            channel.index_playing += 1;

            let track_handle = channel.playing_track().await;

            match track_handle {
                Some(handle) => {
                    let shared_state = Arc::downgrade(&shared_arc);
                    let call_shared = channel.call.as_ref().map(Arc::downgrade)?;

                    let _ = handle.add_event(
                        Event::Track(TrackEvent::End),
                        OnEnd {
                            ctx: self.ctx.clone(),
                            msg: self.msg.clone(),
                            guild_id: self.guild_id.clone(),
                            call: call_shared,
                            shared_state: shared_state.clone(),
                        },
                    );

                    let _ = handle.add_event(
                        songbird::Event::Track(TrackEvent::Playable),
                        OnPlayable {
                            ctx: self.ctx.clone(),
                            msg: self.msg.clone(),
                            guild_id: self.guild_id.clone(),
                            shared_state: shared_state.clone(),
                        },
                    );
                }
                None => {
                    let _ = serenity_utils::send_embed(
                        &self.ctx,
                        &self.msg,
                        "Next up is missing or unavailable",
                        0xFF0000,
                    )
                    .await;
                }
            }
        }
        None
    }
}

#[async_trait]
impl EventHandler for OnPlayable {
    async fn act(&self, _e_ctx: &EventContext<'_>) -> Option<Event> {
        let shared_arc = self.shared_state.upgrade()?;
        let mut shared = shared_arc.lock().await;
        let guild = shared.music_sessions.get_mut(&self.guild_id)?;

        let channel = &mut guild.voice_state;

        if let Some((_, metadata)) = &channel.now_playing {
            if let Some(meta) = metadata {
                let title = meta.title.as_deref().unwrap_or("Unknown");

                let _ = serenity_utils::send_embed(
                    &self.ctx,
                    &self.msg,
                    &format!("🎵  Started playing {}", title),
                    0x6C757D,
                )
                .await;
            }
        }

        None
    }
}

#[async_trait]
impl EventHandler for OnDisconnect {
    async fn act(&self, _e_ctx: &EventContext<'_>) -> Option<Event> {
        commands::voice::leave(self.ctx.clone(), self.msg.clone()).await;
        None
    }
}
