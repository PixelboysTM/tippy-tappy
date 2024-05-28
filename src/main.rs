use poise::CreateReply;
use serenity::all::{
    Colour, CreateEmbed, CreateEmbedAuthor, CreateSelectMenu, CreateSelectMenuKind, Emoji, Message,
    MessageBuilder, User,
};
use serenity::async_trait;
use serenity::builder::{CreateMessage, CreateSelectMenuOption};
use serenity::futures::lock::Mutex;
use serenity::prelude::*;
use std::env;
use std::sync::Arc;
use std::time::Duration;

struct Team {
    name: String,
    flag: String,
    iso: String,
}
struct DataInter {
    teams: Vec<Team>,
}

struct Data {
    inter: Arc<Mutex<DataInter>>,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type PoiseContext<'a> = poise::Context<'a, Data, Error>;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                eprintln!("Error sending message: {why:?}");
            }
            let m = msg
                .channel_id
                .send_message(
                    &ctx.http,
                    CreateMessage::new()
                        .content("Select your favorite animal")
                        .select_menu(
                            CreateSelectMenu::new(
                                "animal_select",
                                CreateSelectMenuKind::String {
                                    options: vec![
                                        CreateSelectMenuOption::new("Cat", "Cat"),
                                        CreateSelectMenuOption::new("Dog", "Dog"),
                                        CreateSelectMenuOption::new("Crab", "Crab"),
                                    ],
                                },
                            )
                            .custom_id("animal_select")
                            .placeholder("No animal seleced"),
                        ),
                )
                .await
                .unwrap();

            let i = match m
                .await_component_interaction(&ctx.shard)
                .timeout(Duration::from_secs(60 * 3))
                .await
            {
                Some(x) => x,
                None => {
                    m.reply(&ctx.http, "Timed out").await.unwrap();
                    return;
                }
            };
        }
    }
}

#[poise::command(slash_command, prefix_command)]
async fn age(
    ctx: PoiseContext<'_>,
    #[description = "Select User"] user: Option<User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
async fn get_teams(ctx: PoiseContext<'_>) -> Result<(), Error> {
    let r = ctx
        .send(
            CreateReply::default().reply(true).embed(
                CreateEmbed::new()
                    .fields(
                        ctx.data()
                            .inter
                            .lock()
                            .await
                            .teams
                            .iter()
                            .map(|t| (format!("{} ({}) {}", t.name, t.iso, t.flag), "", false)),
                    )
                    .description("All available countries")
                    .title("Country List")
                    .color(Colour::DARK_ORANGE)
                    .author(CreateEmbedAuthor::new("Tippy Tappy")),
            ),
        )
        .await;

    if let Err(why) = r {
        println!("{why:?}");
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
async fn add_team(
    ctx: PoiseContext<'_>,
    #[description = "Name"] name: Option<String>,
    #[description = "ISO3"] iso: Option<String>,
    #[description = "Flag"] emoji: Option<String>,
) -> Result<(), Error> {
    if let (Some(name), Some(iso), Some(emoji)) = (name, iso, emoji) {
        let mut r = ctx.data().inter.lock().await;
        r.teams.push(Team {
            name,
            iso,
            flag: emoji,
        });
        ctx.reply("Succesful").await.unwrap();
    } else {
        ctx.reply("Failed not all information provided")
            .await
            .unwrap();
    }

    Ok(())
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
            commands: vec![age(), get_teams(), add_team()],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("!".to_string()),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    inter: Arc::new(Mutex::new(DataInter { teams: vec![] })),
                })
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
