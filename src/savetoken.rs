use rspotify::Token;
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use std::{error::Error, fs};

#[derive(Serialize, Deserialize, Debug)]
struct MyToken{
    RSPOTIFY_CLIENT_ID: String,
    RSPOTIFY_CLIENT_SECRET: String,
    RSPOTIFY_REDIRECT_URI: String,
}
 
impl MyToken{

    
    pub fn from_json() -> Result<Self, Box<dyn Error>> {

        let _content = match fs::read_to_string("token.json") {
        Ok(content) => content,
        Err(_) => {
            // File doesn't exist, create with default values
            let default_app = crate::Appinfo::default();
            let default_token = MyToken {
                RSPOTIFY_CLIENT_ID: default_app.clientId.clone(),
                RSPOTIFY_CLIENT_SECRET: default_app.clientSecret.clone(),
                RSPOTIFY_REDIRECT_URI: default_app.redirectUri.clone(),
            };
            
            let json = serde_json::to_string_pretty(&default_token)?;
            fs::write("token.json", json)?;
            return Ok(default_token);
        }
    };
    
        let content = fs::read_to_string("token.json")?;
        let token: MyToken = from_str(&content)?;
        Ok(token)
    }

    pub fn save_to_json(&self) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string_pretty(&self)?;
        fs::write("token.json", json)?;
        Ok(())
    }

}
    