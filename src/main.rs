use chrono::{DateTime, Utc};
use poise::CreateReply;
use serenity::all::{
    Colour, CreateEmbed, CreateEmbedAuthor, CreateSelectMenu, CreateSelectMenuKind, Message, User,
};
use serenity::async_trait;
use serenity::builder::{CreateMessage, CreateSelectMenuOption};
use serenity::futures::lock::{Mutex, MutexGuard};
use serenity::prelude::*;
use std::env;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::time::Duration;

struct Team {
    name: String,
    flag: String,
    iso: String,
}

struct Game {
    name: String,
    short: String,
    team1_iso: String,
    team2_iso: String,
    start_time: chrono::DateTime<chrono::Local>,
    result: Option<(u16, u16, String)>,
}

struct DataInter {
    teams: Vec<Team>,
    games: Vec<Game>,
}

struct Data {
    inter: Arc<Mutex<DataInter>>,
}

struct SaveGuard<'a>(MutexGuard<'a, DataInter>);

impl<'a> Deref for SaveGuard<'a> {
    type Target = MutexGuard<'a, DataInter>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> DerefMut for SaveGuard<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Drop for SaveGuard<'_> {
    fn drop(&mut self) {
        println!("Save!!!");
    }
}

impl Data {
    async fn lock(&self) -> SaveGuard {
        let g = self.inter.lock().await;
        SaveGuard(g)
    }
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type PoiseContext<'a> = poise::Context<'a, Data, Error>;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    // async fn message(&self, ctx: Context, msg: Message) {
    //     if msg.content == "!ping" {
    //         if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
    //             eprintln!("Error sending message: {why:?}");
    //         }
    //         let m = msg
    //             .channel_id
    //             .send_message(
    //                 &ctx.http,
    //                 CreateMessage::new()
    //                     .content("Select your favorite animal")
    //                     .select_menu(
    //                         CreateSelectMenu::new(
    //                             "animal_select",
    //                             CreateSelectMenuKind::String {
    //                                 options: vec![
    //                                     CreateSelectMenuOption::new("Cat", "Cat"),
    //                                     CreateSelectMenuOption::new("Dog", "Dog"),
    //                                     CreateSelectMenuOption::new("Crab", "Crab"),
    //                                 ],
    //                             },
    //                         )
    //                         .custom_id("animal_select")
    //                         .placeholder("No animal seleced"),
    //                     ),
    //             )
    //             .await
    //             .unwrap();

    //         let i = match m
    //             .await_component_interaction(&ctx.shard)
    //             .timeout(Duration::from_secs(60 * 3))
    //             .await
    //         {
    //             Some(x) => x,
    //             None => {
    //                 m.reply(&ctx.http, "Timed out").await.unwrap();
    //                 return;
    //             }
    //         };
    //     }
    // }
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

#[poise::command(slash_command, prefix_command, required_permissions = "SEND_MESSAGES")]
async fn get_teams(ctx: PoiseContext<'_>) -> Result<(), Error> {
    let r = ctx
        .send(
            CreateReply::default().reply(true).embed(
                CreateEmbed::new()
                    .fields(
                        ctx.data()
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
#[poise::command(slash_command, prefix_command, required_permissions = "ADMINISTRATOR")]
async fn add_team(
    ctx: PoiseContext<'_>,
    #[description = "Name"] name: String,
    #[description = "ISO3"] iso: String,
    #[description = "Flag"] emoji: String,
) -> Result<(), Error> {
    let mut r = ctx.data().lock().await;
    r.teams.push(Team {
        name,
        iso,
        flag: emoji,
    });
    ctx.reply("Succesful").await.unwrap();

    Ok(())
}

#[poise::command(slash_command, prefix_command, required_permissions = "ADMINISTRATOR")]
async fn add_game(
    ctx: PoiseContext<'_>,
    #[description = "Name"] name: String,
    #[description = "Kürzel"] short: String,
    #[description = "Team 1"] team1: String,
    #[description = "Team 2"] team2: String,
    #[description = "Anpfiff"] start_time: String,
) -> Result<(), Error> {
    let mut d = ctx.data().lock().await;
    d.teams
        .iter()
        .find(|t| t.iso == team1)
        .ok_or("Team 1 not present")?;
    d.teams
        .iter()
        .find(|t| t.iso == team2)
        .ok_or("Team 1 not present")?;

    let t = chrono::NaiveDateTime::parse_from_str(&start_time, "%Y %m %d %H:%M")?;

    d.games.push(Game {
        name,
        short,
        team1_iso: team1,
        team2_iso: team2,
        result: None,
        start_time: DateTime::from_naive_utc_and_offset(
            t,
            chrono::offset::FixedOffset::west_opt(0 * 3600).unwrap(),
        ),
    });

    ctx.reply("Succesful").await.unwrap();

    Ok(())
}

#[poise::command(slash_command, prefix_command, required_permissions = "SEND_MESSAGES")]
async fn list_games(ctx: PoiseContext<'_>) -> Result<(), Error> {
    let r = ctx
        .send(
            CreateReply::default().reply(true).embed(
                CreateEmbed::new()
                    .fields(ctx.data().lock().await.games.iter().map(|g| {
                        (
                            format!(
                                "{} ({}) {}",
                                g.name,
                                g.short,
                                g.start_time.format("%d/%m/%Y %H:%M")
                            ),
                            format!(
                                "{} vs. {} | {}:{} {}",
                                g.team1_iso,
                                g.team2_iso,
                                g.result
                                    .as_ref()
                                    .map(|r| r.0.to_string())
                                    .unwrap_or("-".to_string()),
                                g.result
                                    .as_ref()
                                    .map(|r| r.1.to_string())
                                    .unwrap_or("-".to_string()),
                                g.result
                                    .as_ref()
                                    .map(|r| r.2.clone())
                                    .unwrap_or("".to_string()),
                            ),
                            false,
                        )
                    }))
                    .description("All available games")
                    .title("Game List")
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

#[poise::command(slash_command, prefix_command, required_permissions = "ADMINISTRATOR")]
async fn add_score(
    ctx: PoiseContext<'_>,
    #[description = "Das Kürzel des Spiels"] short: String,
    #[description = "Die Tore von Team 1"] team1_score: u16,
    #[description = "Die Tore von Team 2"] team2_score: u16,
    #[description = "Extra informationen zum Spiel"] msg: Option<String>,
) -> Result<(), Error> {
    let mut d = ctx.data().lock().await;
    let g = d
        .games
        .iter_mut()
        .find(|g| g.short == short)
        .ok_or("Kürzel gehört zu keinem Spiel")?;

    g.result = Some((team1_score, team2_score, msg.unwrap_or(String::new())));

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
            commands: vec![
                age(),
                get_teams(),
                add_team(),
                add_game(),
                list_games(),
                add_score(),
            ],
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
                    inter: Arc::new(Mutex::new(DataInter {
                        teams: vec![],
                        games: vec![],
                    })),
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
