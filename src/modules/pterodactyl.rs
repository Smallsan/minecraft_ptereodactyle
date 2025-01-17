use futures_util::{SinkExt, StreamExt};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, Context, CreateMessage, CreateThread};
use std::{clone, error::Error};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[derive(Debug, Deserialize)]
struct WebSocketResponse {
    data: WebSocketData,
}

#[derive(Debug, Deserialize)]
struct WebSocketData {
    token: String,
    socket: String,
}

#[derive(Clone)]
pub struct PterodactylClient {
    api_key: String,
    base_url: String,
    server_id: String,
}

impl PterodactylClient {
    pub fn new(api_key: String, base_url: String, server_id: String) -> Self {
        Self {
            api_key,
            base_url,
            server_id,
        }
    }

    pub async fn connect_websocket(
        &self,
        discord_ctx: Context,
        channel_id: ChannelId,
    ) -> Result<(), Box<dyn Error>> {
        let client = Client::new();
        let url = format!(
            "{}/api/client/servers/{}/websocket",
            self.base_url, self.server_id
        );

        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Accept", "application/vnd.pterodactyl.v1+json")
            .send()
            .await?;

        if response.status() != 200 {
            let error_text = response.text().await?;
            eprintln!("Failed to get websocket details: {}", error_text);
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to get websocket details",
            )));
        }

        let ws_response = response.json::<WebSocketResponse>().await?;
        println!("WebSocket Details: {:?}", ws_response);

        use tokio_tungstenite::tungstenite::client::IntoClientRequest;
        let mut request = ws_response.data.socket.clone().into_client_request()?;
        request
            .headers_mut()
            .insert("Origin", "https://control.sparkedhost.us".parse().unwrap());

        let (ws_stream, _) = connect_async(request).await?;
        let (mut write, mut read) = ws_stream.split();

        let auth_message = serde_json::json!({
            "event": "auth",
            "args": [ws_response.data.token]
        });
        write
            .send(Message::Text(auth_message.to_string().into()))
            .await?;
        println!("WebSocket authenticated.");

        let ip_regex = Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").unwrap();

        // Handle incoming messages
        while let Some(msg) = read.next().await {
            match msg {
                Ok(msg) => {
                    if let Ok(text) = msg.into_text() {
                        let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
                        if let Some(event) = parsed.get("event") {
                            match event.as_str().unwrap() {
                                "stats" => {
                                    // Ignore stats data
                                }
                                "console output" => {
                                    if let Some(args) = parsed.get("args") {
                                        if let Some(message) = args[0].as_str() {
                                            // Extract the actual message content by removing timestamps and thread info
                                            if let Some(content_start) = message.rfind("]: ") {
                                                let content = &message[content_start + 3..];

                                                // Remove IP addresses from the content
                                                let filtered_content =
                                                    ip_regex.replace_all(content, "[REDACTED]");

                                                // Send the filtered message content to the channel
                                                channel_id
                                                    .send_message(
                                                        &discord_ctx.http,
                                                        CreateMessage::new()
                                                            .content(&filtered_content),
                                                    )
                                                    .await?;
                                            }
                                        }
                                    }
                                }
                                _ => {
                                    println!("Received: {}", text);
                                }
                            }
                        }
                    }
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }

        Ok(())
    }
}
