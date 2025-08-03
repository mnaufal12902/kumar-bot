use reqwest::Client;
use serenity::all::{ChannelId, GuildId};
use songbird::{
    Call,
    input::{
        AuxMetadata, Input, YoutubeDl,
        codecs::{CODEC_REGISTRY, PROBE},
    },
    tracks::TrackHandle,
};
use std::{collections::HashMap, sync::Arc};
use symphonia::core::{codecs::CODEC_TYPE_OPUS, probe::Probe};
use tokio::sync::Mutex;

pub struct BotMusicState {
    pub music_sessions: HashMap<GuildId, GuildMusicSession>,
}

pub struct GuildMusicSession {
    pub guild_id: GuildId,
    pub channel_id: ChannelId,
    pub voice_state: VoiceChannelMusicState,
}

pub struct VoiceChannelMusicState {
    pub call: Option<Arc<Mutex<Call>>>,
    pub queue: Vec<QueuedTrack>,
    pub now_playing: Option<(TrackHandle, Option<AuxMetadata>)>,
    pub index_playing: usize,
    pub volume: f32,
}

pub struct QueuedTrack {
    pub url: String,
    pub resolved: Option<Input>,
}

impl BotMusicState {
    pub fn new() -> Self {
        Self {
            music_sessions: HashMap::new(),
        }
    }
}

impl GuildMusicSession {
    pub fn new(call: Option<Arc<Mutex<Call>>>, guild_id: GuildId, channel_id: ChannelId) -> Self {
        Self {
            guild_id,
            channel_id,
            voice_state: VoiceChannelMusicState::new(call),
        }
    }
}

impl VoiceChannelMusicState {
    pub fn new(call: Option<Arc<Mutex<Call>>>) -> Self {
        Self {
            call,
            queue: Vec::new(),
            now_playing: None,
            index_playing: 0,
            volume: 1.0,
        }
    }

    pub async fn playing_track(&mut self) -> Option<TrackHandle> {
        let call = &mut self.call.clone()?;

        let mut handler = call.lock().await;
        let current_track = self.get_current_track()?;

        if let Some(mut input) = current_track.take_input() {
            let metadata: Option<AuxMetadata> = match input.aux_metadata().await {
                Ok(meta) => Some(meta),
                Err(_) => None,
            };

            let handle = handler.play_input(input);

            let _ = handle.set_volume(self.volume);
            self.now_playing = Some((handle.clone(), metadata));

            Some(handle)
        } else {
            None
        }
    }

    pub async fn add_track(&mut self, url: String, client: Client) -> usize {
        let mut track = QueuedTrack {
            url: url.clone(),
            resolved: None,
        };

        match self.now_playing {
            Some(_) => track.preload(client, true).await,
            None => track.preload(client, false).await,
        };

        self.queue.push(track);

        self.queue.iter().position(|q| q.url == url).unwrap_or(0) + 1
    }

    pub fn get_current_track(&mut self) -> Option<&mut QueuedTrack> {
        self.queue.get_mut(self.index_playing)
    }
}

impl QueuedTrack {
    pub async fn preload(&mut self, client: Client, is_preload: bool) {
        if self.resolved.is_none() {
            let input = Input::from(YoutubeDl::new(client, self.url.clone()).user_args(vec![
                "--no-playlist".into(),
                "-f".into(),
                "bestaudio[acodec=opus]/bestaudio".into(),
            ]));

            match is_preload {
                true => match input.make_playable_async(&CODEC_REGISTRY, &PROBE).await {
                    Ok(i) => {
                        tracing::info!("Preload song");
                        self.resolved = Some(i)
                    }
                    Err(err) => {
                        tracing::error!("Error : {}", err);
                        self.resolved = None
                    }
                },
                false => self.resolved = Some(input),
            }
        }
    }

    pub fn take_input(&mut self) -> Option<Input> {
        tracing::info!("Taking input");
        self.resolved.take()
    }
}
