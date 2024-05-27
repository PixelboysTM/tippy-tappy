use std::env;
use std::time::Duration;
use serenity::all::{CreateSelectMenu, CreateSelectMenuKind, Message, MessageBuilder};
use serenity::async_trait;
use serenity::builder::{CreateMessage, CreateSelectMenuOption};
use serenity::prelude::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                eprintln!("Error sending message: {why:?}");
            }
            let m = msg.channel_id.send_message(&ctx.http, CreateMessage::new().content("Select your favorite animal").select_menu(CreateSelectMenu::new("animal_select", CreateSelectMenuKind::String {
                options: vec![
                    CreateSelectMenuOption::new("Cat", "Cat"),
                    CreateSelectMenuOption::new("Dog", "Dog"),
                    CreateSelectMenuOption::new("Crab", "Crab"),
                ]
            }).custom_id("animal_select").placeholder("No animal seleced"))).await.unwrap();
            
            let i = match m.await_component_interaction(&ctx.shard).timeout(Duration::from_secs(60 * 3)).await {
                Some(x) => x,
                None => {
                    m.reply(&ctx.http, "Timed out").await.unwrap();
                    return;
                }
            };


        }
    }


}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Must be set");
    let intents = GatewayIntents::GUILD_MESSAGES |GatewayIntents::DIRECT_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents).event_handler(Handler).await.expect("Err creating client");
    if let Err(why) = client.start().await {
        eprintln!("Client error: {why:?}");
    }
}
