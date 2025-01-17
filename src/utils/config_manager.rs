use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::Path,
};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Config {
    pub notify_on_join: String,
    pub notify_on_leave: String,
    pub notification_channel_id: String,
    pub apollo_server_id: String,
}

pub fn get_config() -> Config {
    let folder_name = Path::new("config");
    let file_name = "config.json";
    if !folder_name.exists() {
        fs::create_dir(folder_name).expect("Failed to create config directory");
    }
    let path = folder_name.join(file_name);
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&path)
        .expect("Failed to open config file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read config file");
    if contents.is_empty() {
        println!(
            "Config file is empty, creating default config at {}",
            path.display()
        );
        let default_config = Config {
            notify_on_join: "True".to_string(),
            notify_on_leave: "True".to_string(),
            notification_channel_id: "".to_string(),
            apollo_server_id: "".to_string(),
        };
        let json = serde_json::to_string_pretty(&default_config)
            .expect("Failed to serialize default config");
        file.write_all(json.as_bytes())
            .expect("Failed to write default config to file");
        return default_config;
    }
    serde_json::from_str(&contents).expect("Failed to deserialize config file")
}
