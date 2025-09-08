use rspotify::{
    AuthCodeSpotify, ClientResult, Credentials, OAuth,
    model::{AdditionalType, Country, Market},
    prelude::*,
    scopes,
};
use std::io;

#[tokio::main]
async fn main() {
    // You can use any logger for debugging.
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
        "user-modify-playback-state"
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
        io::stdin().read_line(&mut input);
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

            "exit" => {
                break;
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
        let market = Market::Country(Country::Spain);
        let additional_types = [AdditionalType::Episode];
        let artists = self
            .spotify
            .current_playing(Some(market), Some(&additional_types))
            .await;

        println!("Response: {artists:?}");
        Ok(())
    }
}
