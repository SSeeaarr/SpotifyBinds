use egui::{Context, Ui};
use rspotify::{
    AuthCodeSpotify, ClientError, ClientResult, Config, Credentials, OAuth,
    model::{AdditionalType, Country, FullTrack, Market, PlayableItem, device, track},
    prelude::*,
    scopes,
};
use std::io;


include!("savetoken.rs");

async fn spotifyinit() -> Option<AuthCodeSpotify> {
    // Set RSPOTIFY_CLIENT_ID and RS POTIFY_CLIENT_SECRET in an .env file (after
    // enabling the `env-file` feature) or export them manually:
    //
    // export RSPOTIFY_CLIENT_ID="your client_id"
    // export RSPOTIFY_CLIENT_SECRET="secret"
    //
    // These will then be read with `from_env`.
    //
    // Otherwise, set client_id and client_secret explictly:
    //
    // ```
    // let creds = Credentials::new("my-client-id", "my-client-secret");
    // ```
    //let creds = Credentials::from_env().unwrap();

    let token = MyToken::from_json().ok()?;
    let client_id = &token.RSPOTIFY_CLIENT_ID;
    let client_secret = &token.RSPOTIFY_CLIENT_SECRET;
    let redirect_uri = &token.RSPOTIFY_REDIRECT_URI;

    if client_id.is_empty() || client_secret.is_empty() || redirect_uri.is_empty() {
        println!("Spotify credentials are missing or empty");
        return None;
    }

    let creds = Credentials::new(&client_id, &client_secret);

    // Same for RSPOTIFY_REDIRECT_URI. You can also set it explictly:
    //
    // ```
    // let oauth = OAuth {
    //     redirect_uri: "http://localhost:8888/callback".to_string(),
    //     scopes: scopes!("user-read-recently-played"),
    //     ..Default::default(),
    // };
    // ```
    let oauth = OAuth {
        redirect_uri: redirect_uri.clone(),
        scopes: scopes!(
            "user-read-currently-playing",
            "user-modify-playback-state",
            "user-read-playback-state"
        ),
        ..Default::default()
    };

    let cache_path = if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            dir.join(".spotify_token_cache.json")
        } else {
            std::path::PathBuf::from(".spotify_token_cache.json")
        }
    } else {
        std::path::PathBuf::from(".spotify_token_cache.json")
    };

    let config = Config {
        token_cached: true,
        token_refreshing: true,
        cache_path,
        ..Default::default()
    };

    let spotify = AuthCodeSpotify::with_config(creds, oauth, config);

    // Obtaining the access token
    let url = spotify.get_authorize_url(false).ok()?;

    // This function requires the `cli` feature enabled.

    spotify.prompt_for_token(&url).await.ok()?;

    Some(spotify)

    // Running the requests
    /*

    let mut input = String::new();
    let coreloop: bool = true;
    println!("listening for input...");
    while coreloop {
        let _ = io::stdin().read_line(&mut input);
        match input.trim() {
            "next" => {
                SpotifyClient {
                    spotify: spotify.clone(),
                }
                .next_track(None)
                .await
                .unwrap();
                println!("Skipped to next track");
            }

            "previous" => {
                SpotifyClient {
                    spotify: spotify.clone(),
                }
                .previous_track(None)
                .await
                .unwrap();
                println!("Went to previous track");
            }

            "info" => {
                SpotifyClient {
                    spotify: spotify.clone(),
                }
                .song_info(None)
                .await
                .unwrap();
            }

            "current" => {
                SpotifyClient {
                    spotify: spotify.clone(),
                }
                .current_song()
                .await
                .unwrap();
            }

            "play" => {
                SpotifyClient {
                    spotify: spotify.clone(),
                }
                .play(None)
                .await
                .unwrap();
            }

            "pause" => {
                SpotifyClient {
                    spotify: spotify.clone(),
                }
                .pause(None)
                .await
                .unwrap();
            }

            "toggle" => {
                SpotifyClient {
                    spotify: spotify.clone(),
                }
                .toggle_playback(None)
                .await
                .unwrap();
            }

            "exit" => {
                break;
            }

            "current queue" => {
                SpotifyClient {
                    spotify: spotify.clone(),
                }
                .current_queue()
                .await
                .unwrap();
            }

            _ => {
                println!("Unknown command.");
            }
        }

        input.clear();
    }
    */
}

struct SpotifyClient {
    spotify: AuthCodeSpotify,
}

impl SpotifyClient {
    async fn pause(&self, device_id: Option<&str>) -> ClientResult<()> {
        if let Err(e) = self.spotify.pause_playback(device_id).await {
            println!("Already paused! : {}", e);
            return Ok(());
        }
        Ok(())
    }

    async fn get_volume(&self, device_id: Option<&str>) -> ClientResult<Option<u32>> {
        let playback = self.spotify.current_playback(None, None::<Vec<_>>).await?;
        if let Some(pb) = playback {
            if let Some(volume) = pb.device.volume_percent {
                return Ok(Some(volume));
            }
        }
        Ok(None)
    }

    async fn mute(&self, device_id: Option<&str>) -> ClientResult<()> { //this doesnt work
        self.spotify.volume(0, device_id).await?;
        Ok(())
    }

    async fn volup(&self, device_id: Option<&str>, incamt: u32) -> ClientResult<()> {
        let currentvol = match self.get_volume(device_id).await? {
            Some(vol) => vol,
            None => {
                println!("Could not get current volume");
                return Ok(());
            }
        };

        let new_vol = currentvol.saturating_add(incamt).min(100);
        self.spotify.volume(new_vol as u8, device_id).await?;
        Ok(())
    }

    async fn voldown(&self, device_id: Option<&str>, decamt: u32) -> ClientResult<()> {
        let currentvol = match self.get_volume(device_id).await? {
            Some(vol) => vol,
            None => {
                println!("Could not get current volume");
                return Ok(());
            }
        };

        let new_vol = currentvol.saturating_sub(decamt);
        self.spotify.volume(new_vol as u8, device_id).await?;
        Ok(())
    }

    async fn play(&self, device_id: Option<&str>) -> ClientResult<()> {
        // Try to resume existing playback first
        if self.spotify.resume_playback(device_id, None).await.is_ok() {
            return Ok(());
        }
        
        // If that fails, try to resume on an available device
        match self.get_available_device().await {
            Ok(Some(dev_id)) => {
                let _ = self.spotify.resume_playback(Some(&dev_id), None).await;
                Ok(())
            }
            _ => {
                println!("Could not start playback: no active device");
                Ok(())
            }
        }
    }

    async fn toggle_playback(&self, device_id: Option<&str>) -> ClientResult<()> {
        let playback = self.spotify.current_playback(None, None::<Vec<_>>).await?;

        if let Some(context) = playback {
            if context.is_playing {
                self.spotify.pause_playback(device_id).await?;
            } else {
                self.spotify.resume_playback(device_id, None).await?;
            }
        } else {
            // No active playback - try to start it on an available device
            match self.get_available_device().await {
                Ok(Some(dev_id)) => {
                    let _ = self.spotify.resume_playback(Some(&dev_id), None).await;
                }
                _ => {
                    println!("Could not start playback: no active device");
                }
            }
        }

        Ok(())
    }

    async fn next_track(&self, device_id: Option<&str>) -> ClientResult<()> {
        self.spotify.next_track(device_id).await?;
        Ok(())
    }

    async fn previous_track(&self, device_id: Option<&str>) -> ClientResult<()> {
        self.spotify.previous_track(device_id.as_deref()).await?;
        Ok(())
    }

    async fn get_available_device(&self) -> ClientResult<Option<String>> {
        let devices = self.spotify.device().await?;
        Ok(devices.first().and_then(|d| d.id.clone()))
    }

    async fn song_info(&self, device_id: Option<&str>) -> ClientResult<()> {
        let market = Market::Country(Country::UnitedStates);
        let additional_types = [AdditionalType::Episode];
        let artists = self
            .spotify
            .current_playing(Some(market), Some(&additional_types))
            .await;

        println!("Response: {artists:?}");
        Ok(())
    }

    async fn current_queue(&self) -> ClientResult<()> {
        match self.spotify.current_user_queue().await {
            Ok(res) => {
                for song in &res.queue {
                    if let PlayableItem::Track(track) = song {
                        let artist_names: Vec<&str> = track
                            .artists
                            .iter()
                            .map(|artist| artist.name.as_str())
                            .collect();

                        println!("{:#?} -- {:#?}", track.name, artist_names);
                    }
                }
                if res.queue.is_empty() {
                    println!("Queue is empty")
                }

                Ok(())
            }
            Err(e) => {
                println!("Error fetching queue: {:?}", e);
                Err(e)
            }
        }
    }

    async fn current_song(&self) -> ClientResult<()> {
        let song = self
            .spotify
            .current_playing(None, None::<Vec<_>>)
            .await?
            .unwrap();
        if let Some(PlayableItem::Track(track)) = song.item {
            let artist_names: Vec<&str> = track
                .artists
                .iter()
                .map(|artist| artist.name.as_str())
                .collect();
            println!(
                "Song: {:?} on album: {:?} Artist: {:?}",
                track.name, track.album.name, artist_names
            )
        }
        Ok(())
    }
}
