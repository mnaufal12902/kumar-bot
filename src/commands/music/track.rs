use std::sync::Arc;

use serenity::{client::Context, model::prelude::Message};
use songbird::{Event, TrackEvent};

use crate::{
    api::{spotify::get_track_by_id, youtube::search_youtube},
    bot::{HttpKey, MusicStateKey},
    commands::{
        self,
        music::event::{OnEnd, OnPlayable},
    },
    models::spotify::SpotifyTrackItem,
    token::registry::TokenRegistry,
    utils::serenity_utils,
};

#[derive(Debug)]
pub enum MediaTrack {
    Spotify(SpotifyTrackItem),
    // Future: YouTube(YouTubeTrackItem), SoundCloud(...), etc.
}

pub async fn run(ctx: Context, msg: Message, registry: TokenRegistry) {
    if let Some(guild_id) = msg.guild_id {
        let args = msg.content.strip_prefix("pb!play").unwrap_or("").trim();
        tracing::debug!("Args: {:?}", args);

        let media_result: Option<MediaTrack> = match parse_media(args, registry).await {
            Ok(Some(media)) => {
                tracing::debug!("Media track loaded: {:?}", media);
                Some(media)
            }
            Ok(None) => {
                serenity_utils::send_embed(
                    &ctx,
                    &msg,
                    "Could not find the provided song",
                    0xFF0000,
                )
                .await;
                None
            }
            Err(e) => {
                tracing::error!("Error while fetching media: {:?}", e);
                serenity_utils::send_embed(&ctx, &msg, "Failed to load track 😞", 0xFF0000).await;
                None
            }
        };

        match media_result {
            Some(MediaTrack::Spotify(track_item)) => {
                let http_client = {
                    let data = ctx.data.read().await;
                    data.get::<HttpKey>()
                        .cloned()
                        .expect("Guaranteed to exist in the typemap.")
                };

                let query = format!(
                    "{} {}",
                    track_item.name,
                    track_item
                        .artists
                        .get(0)
                        .map(|artist| artist.name.clone())
                        .unwrap_or("".to_string())
                );

                let url = match search_youtube(&query).await {
                    Ok(results) => {
                        let video_id = results
                            .into_iter()
                            .next()
                            .map(|res| res.video_id)
                            .unwrap_or_default();
                        let url = format!("https://www.youtube.com/watch?v={}", video_id);
                        url
                    }
                    Err(e) => {
                        tracing::error!("Failed to fetch YouTube source: {:?}", e);
                        serenity_utils::send_embed(
                            &ctx,
                            &msg,
                            "Failed to get audio source 😞",
                            0xFF0000,
                        )
                        .await;
                        return;
                    }
                };

                // Force bot to join channel
                commands::voice::join(ctx.clone(), msg.clone()).await;

                let music_state = ctx
                    .data
                    .read()
                    .await
                    .get::<MusicStateKey>()
                    .unwrap()
                    .clone();
                let mut state = music_state.lock().await;

                if let Some(channel_state) = &mut state
                    .music_sessions
                    .get_mut(&guild_id)
                    .and_then(|session| Some(&mut session.voice_state))
                {
                    channel_state.add_track(url, http_client.clone()).await;

                    if let Some(call) = channel_state.call.clone() {
                        if channel_state.now_playing.is_none() {
                            let track_handle = channel_state.playing_track().await;

                            match track_handle {
                                Some(handle) => {
                                    let shared_state = Arc::downgrade(&music_state);
                                    let call_shared = Arc::downgrade(&call);

                                    let _ = handle.add_event(
                                        Event::Track(TrackEvent::End),
                                        OnEnd {
                                            ctx: ctx.clone(),
                                            msg: msg.clone(),
                                            guild_id: guild_id.clone(),
                                            call: call_shared,
                                            shared_state: shared_state.clone(),
                                        },
                                    );

                                    let _ = handle.add_event(
                                        songbird::Event::Track(TrackEvent::Playable),
                                        OnPlayable {
                                            ctx: ctx.clone(),
                                            msg: msg.clone(),
                                            guild_id: guild_id.clone(),
                                            shared_state: shared_state.clone(),
                                        },
                                    );
                                }
                                None => {
                                    serenity_utils::send_embed(
                                        &ctx,
                                        &msg,
                                        "Failed to playing audio😞",
                                        0xFF0000,
                                    )
                                    .await;
                                }
                            }
                        };
                    }
                }
            }
            None => return,
        }
    }
}

pub async fn pause(ctx: Context, msg: Message) {
    if let Some(guild_id) = msg.guild_id {
        let music_state = ctx
            .data
            .read()
            .await
            .get::<MusicStateKey>()
            .unwrap()
            .clone();

        let mut state = music_state.lock().await;
        let session = match state.music_sessions.get_mut(&guild_id) {
            Some(s) => s,
            None => {
                return;
            }
        };

        let channel_state = &mut session.voice_state;

        if let Some((track_handle, _)) = &mut channel_state.now_playing {
            let _ = track_handle.pause();

            serenity_utils::send_embed(
                &ctx,
                &msg,
                &format!("⏸️ The current song has been paused"),
                0x6C757D,
            )
            .await;
        } else {
            return;
        }
    } else {
        serenity_utils::send_embed(&ctx, &msg, "Failed to get server😞", 0xFF0000).await;
    };
}

pub async fn resume(ctx: Context, msg: Message) {
    if let Some(guild_id) = msg.guild_id {
        let music_state = ctx
            .data
            .read()
            .await
            .get::<MusicStateKey>()
            .unwrap()
            .clone();

        let mut state = music_state.lock().await;
        let session = match state.music_sessions.get_mut(&guild_id) {
            Some(s) => s,
            None => {
                return;
            }
        };

        let channel_state = &mut session.voice_state;

        if let Some((track_handle, _)) = &mut channel_state.now_playing {
            let _ = track_handle.play();

            serenity_utils::send_embed(&ctx, &msg, &format!("▶️ Back to the music"), 0x6C757D)
                .await;
        } else {
            return;
        }
    } else {
        serenity_utils::send_embed(&ctx, &msg, "Failed to get server😞", 0xFF0000).await;
    };
}

pub async fn skip(ctx: Context, msg: Message) {
    let user_id = msg.author.id;
    let user_info = user_id.to_user(&ctx.http).await;

    if let Some(guild_id) = msg.guild_id {
        let music_state = ctx
            .data
            .read()
            .await
            .get::<MusicStateKey>()
            .unwrap()
            .clone();

        let mut state = music_state.lock().await;
        let session = match state.music_sessions.get_mut(&guild_id) {
            Some(s) => s,
            None => {
                return;
            }
        };

        let channel_state = &mut session.voice_state;

        if let Some((track_handle, metadata)) = &channel_state.now_playing {
            let username = user_info.map(|u| u.name).unwrap_or("unknown".to_string());
            let title = metadata
                .clone()
                .map(|m| m.title.unwrap_or("unknown".to_string()))
                .unwrap();

            let _ = track_handle.stop();

            serenity_utils::send_embed(
                &ctx,
                &msg,
                &format!("{} has been skipped by @{}", title, username),
                0x6C757D,
            )
            .await;
        } else {
            serenity_utils::send_embed(
                &ctx,
                &msg,
                &format!("You're not playing any music"),
                0x6C757D,
            )
            .await;
        }
    }
}

async fn parse_media(args: &str, registry: TokenRegistry) -> anyhow::Result<Option<MediaTrack>> {
    if args.contains("open.spotify.com/track/") {
        if let Some(track_id) = extract_track_id(args) {
            let track = get_track_by_id(track_id, registry).await?;
            return Ok(Some(MediaTrack::Spotify(track)));
        } else {
            tracing::warn!("Could not extract Spotify ID");
            return Ok(None);
        }
    }

    tracing::warn!("Unsupported URL or command: {}", args);
    Ok(None)
}

pub fn extract_track_id(url: &str) -> Option<String> {
    url.split("/track/")
        .nth(1)
        .and_then(|s| s.split('?').next())
        .map(|s| s.to_string())
}
