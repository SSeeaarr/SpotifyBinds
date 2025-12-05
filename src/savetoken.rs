
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use std::{error::Error, fs}; 

#[derive(Serialize, Deserialize, Debug)]
struct MyToken{
    RSPOTIFY_CLIENT_ID: String,
    RSPOTIFY_CLIENT_SECRET: String,
    RSPOTIFY_REDIRECT_URI: String,
}
 
impl MyToken {
    fn path() -> std::path::PathBuf {
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                return dir.join("token.json");
            }
        }
        std::path::PathBuf::from("token.json")
    }

    pub fn from_json() -> Result<Self, Box<dyn Error>> {
        let p = Self::path();
        let content = match fs::read_to_string(&p) {
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
                fs::write(&p, json)?;
                
                
                return Ok(default_token);
            }
        };
        
        let token: MyToken = from_str(&content)?;
        Ok(token)
    }

    pub fn save_to_json(&self) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string_pretty(&self)?;
        fs::write(Self::path(), json)?;
        Ok(())
    }
}

