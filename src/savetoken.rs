use serde::{Deserialize, Serialize};
use serde_json::from_str;
use std::{error::Error, fs};

#[derive(Serialize, Deserialize, Debug)]
struct Token{
    RSPOTIFY_CLIENT_ID: String,
    RSPOTIFY_CLIENT_SECRET: String,
    RSPOTIFY_REDIRECT_URI: String,
}

impl Token{
    
    pub fn from_json() -> Result<Self, Box<dyn Error>> {

        let _content = match fs::read_to_string("token.json") {
        Ok(content) => content,
        Err(_) => {
            // File doesn't exist, create with default values
            let default_token = Token {
                RSPOTIFY_CLIENT_ID: String::new(),
                RSPOTIFY_CLIENT_SECRET: String::new(),
                RSPOTIFY_REDIRECT_URI: String::new(),
            };
            
            let json = serde_json::to_string_pretty(&default_token)?;
            fs::write("token.json", json)?;
            return Ok(default_token);
        }
    };
    
        let content = fs::read_to_string("token.json")?;
        let token: Token = from_str(&content)?;
        Ok(token)
    }

}