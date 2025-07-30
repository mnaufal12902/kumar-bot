use reqwest::Client;
use serenity::all::{ChannelId, GuildId};
use songbird::{
    Call,
    input::{AuxMetadata, Input, YoutubeDl},
    tracks::TrackHandle,
};
use std::{collections::HashMap, sync::Arc};
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

            self.now_playing = Some((handle.clone(), metadata));
            Some(handle)
        } else {
            None
        }
    }

    pub async fn add_track(&mut self, url: String, client: Client) {
        let mut track = QueuedTrack {
            url: url.clone(),
            resolved: None,
        };

        track.preload(client).await;

        self.queue.push(track);
    }

    pub fn get_current_track(&mut self) -> Option<&mut QueuedTrack> {
        self.queue.get_mut(self.index_playing)
    }

    // pub async fn reset_state(&mut self) {
    //     if let Some(call_state) = &self.call {
    //         call_state.lock().await.stop();
    //     };

    //     self.index_playing = 0;
    //     self.now_playing = None;
    //     self.call = None;
    //     self.queue.clear();
    // }
}

impl QueuedTrack {
    pub async fn preload(&mut self, client: Client) {
        if self.resolved.is_none() {
            let input = Input::from(YoutubeDl::new(client, self.url.clone()).user_args(vec![
                "--no-playlist".into(),
                "-f".into(),
                "bestaudio[acodec=opus]/bestaudio".into(),
            ]));
            self.resolved = Some(input);
        }
    }

    pub fn take_input(&mut self) -> Option<Input> {
        tracing::info!("Taking input");
        self.resolved.take()
    }
}
