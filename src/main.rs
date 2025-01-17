mod utils;
use utils::{config_manager::get_config, key_manager::get_key};

mod modules;

use core::panic;

use serenity::{
    all::{ChannelId, Context, EventHandler, GatewayIntents, Message, Ready},
    async_trait, Client,
};


struct Handler {
    pterodactyl_client: modules::pterodactyl::PterodactylClient,
    notification_channel_id: ChannelId,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("Connected as {}", ready.user.name);

        let discord_ctx = ctx.clone();
        let channel_id = self.notification_channel_id;

        let pterodactyl_client = self.pterodactyl_client.clone();
        tokio::spawn(async move {
            if let Err(why) = pterodactyl_client.connect_websocket(discord_ctx, channel_id).await {
                eprintln!("Pterodactyl client error: {:?}", why);
            }
        });
    }
}
#[tokio::main]
async fn main() {
    let key = get_key();
    let discord_api_key = key.discord_api_key;
    let apollo_api_key = key.apollo_api_key;

    let config = get_config();
    let notification_channel_id = config.notification_channel_id.parse().expect("Failed to parse channel ID");
    let apollo_server_id = config.apollo_server_id;
    let notify_on_join = config.notify_on_join;
    let notify_on_leave = config.notify_on_leave;

    let pterodactyl_client = modules::pterodactyl::PterodactylClient::new(
        apollo_api_key,
        "https://control.sparkedhost.us".to_string(),
        apollo_server_id,
    );

    let handler = Handler {
        pterodactyl_client,
        notification_channel_id,
    };

    let mut discord_client = Client::builder(discord_api_key, GatewayIntents::default())
        .event_handler(handler)
        .await
        .expect("Error creating Discord client");

    println!("Starting clients");

    if let Err(why) = discord_client.start().await {
        eprintln!("Discord client error: {:?}", why);
    }
}