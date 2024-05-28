use chrono::{DateTime, Utc};
use poise::CreateReply;
use serenity::all::{ActivityData, Colour, CreateEmbed, CreateEmbedAuthor, CreateSelectMenu, CreateSelectMenuKind, Message, Ready, User};
use serenity::async_trait;
use serenity::builder::{CreateMessage, CreateSelectMenuOption};
use serenity::futures::lock::{Mutex, MutexGuard};
use serenity::prelude::*;
use std::env;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::time::Duration;
use crate::cmds::get_cmds;
use crate::data::{Data, Game, Team};

mod data;
mod cmds;


type Error = Box<dyn std::error::Error + Send + Sync>;
type PoiseContext<'a> = poise::ApplicationContext<'a, Data, Error>;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, data_about_bot: Ready) {
        ctx.set_activity(Some(ActivityData::watching("Fußball")));
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("Must be set");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: get_cmds(),
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("!".to_string()),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data::new())
            })
        })
        .build();

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Err creating client");
    if let Err(why) = client.start().await {
        eprintln!("Client error: {why:?}");
    }
}
