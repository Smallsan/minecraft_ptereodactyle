use std::{
    fs::{self, read, File, OpenOptions},
    io::{Read, Write},
    path::Path,
};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Key {
    pub discord_api_key: String,
    pub apollo_api_key: String,
}

pub fn get_key() -> Key {
    let folder_name = Path::new("config");
    let file_name = "keys.json";
    if !folder_name.exists() {
        fs::create_dir(folder_name).expect("Failed to create config directory");
    }
    let path = folder_name.join(file_name);
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&path)
        .expect("Failed to open key file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read key file");
    if contents.is_empty() {
        println!(
            "Key file is empty, creating default keys at {}",
            path.display()
        );
        let default_keys = Key {
            discord_api_key : "discord_token_here".to_string(),
            apollo_api_key : "apollo_token_here".to_string(),
        };
        let json =
            serde_json::to_string_pretty(&default_keys).expect("Failed to serialize default keys");
        file.write_all(json.as_bytes())
            .expect("Failed to write default keys to file");
        return default_keys;
    }
    serde_json::from_str(&contents).expect("Failed to deserialize key file")
}
