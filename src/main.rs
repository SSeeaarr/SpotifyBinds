use rspotify::{
    model::{AdditionalType, Country, FullTrack, Market, PlayableItem}, prelude::*, scopes, AuthCodeSpotify, ClientResult, Credentials, OAuth
};
use serde_json::value::Index;
use std::io;

include!("hotkeyreg.rs");

#[tokio::main]
async fn main() {
    // You can use any logger for debugging.
    listenforkey().await;
    env_logger::init();

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
    let creds = Credentials::from_env().unwrap();

    // Same for RSPOTIFY_REDIRECT_URI. You can also set it explictly:
    //
    // ```
    // let oauth = OAuth {
    //     redirect_uri: "http://localhost:8888/callback".to_string(),
    //     scopes: scopes!("user-read-recently-played"),
    //     ..Default::default(),
    // };
    // ```
    let oauth = OAuth::from_env(scopes!(
        "user-read-currently-playing",
        "user-modify-playback-state",
        "user-read-playback-state"
    ))
    .unwrap();

    let spotify = AuthCodeSpotify::new(creds, oauth);

    // Obtaining the access token
    let url = spotify.get_authorize_url(false).unwrap();
    
    // This function requires the `cli` feature enabled.
    
    spotify.prompt_for_token(&url).await.unwrap();
    
    

    // Running the requests

    let mut input = String::new();
    let coreloop: bool = true;
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
}

struct SpotifyClient {
    spotify: AuthCodeSpotify,
}



impl SpotifyClient {
    async fn next_track(&self, device_id: Option<&str>) -> ClientResult<()> {
        self.spotify.next_track(device_id).await?;
        Ok(())
    }

    async fn previous_track(&self, device_id: Option<&str>) -> ClientResult<()> {
        self.spotify.previous_track(device_id.as_deref()).await?;
        Ok(())
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
            for song in &res.queue{
                if let PlayableItem::Track(track) = song{
                    
                    println!("{:#?} -- {:#?}", track.name, track.artists);
                }
            }
            
            Ok(())
        },
        Err(e) => {
            println!("Error fetching queue: {:?}", e);
            Err(e)
        }
        }
    }

    async fn current_song(&self) -> ClientResult<()> {
        let song = self.spotify.current_playing(None, None::<Vec<_>>).await?.unwrap();
            if let Some(PlayableItem::Track(track)) = song.item {
                println!("{:?}", track.name)
            }
        Ok(())
    }
}



    

