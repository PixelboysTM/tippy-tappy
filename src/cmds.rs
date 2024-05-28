use chrono::DateTime;
use poise::{Command, CreateReply};
use poise::futures_util::stream::iter;
use poise::serenity_prelude::futures;
use serenity::all::{Colour, CreateEmbed, CreateEmbedAuthor, User};
use serenity::futures::Stream;
use crate::{Error, PoiseContext};
use crate::data::{Bet, Data, Game, Team};
use poise::futures_util::StreamExt;

pub fn get_cmds() -> Vec<Command<Data, Error>> {
     vec![
        age(),
        get_teams(),
        add_team(),
        add_game(),
        list_games(),
        add_score(),
        bet()
    ]
}

#[poise::command(slash_command)]
async fn age(
    ctx: PoiseContext<'_>,
    #[description = "Select User"] user: Option<User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}

#[poise::command(slash_command, required_permissions = "SEND_MESSAGES")]
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
            ).ephemeral(true),
        )
        .await;

    if let Err(why) = r {
        println!("{why:?}");
    }

    Ok(())
}
#[poise::command(slash_command, required_permissions = "ADMINISTRATOR")]
async fn add_team(
    ctx: PoiseContext<'_>,
    #[description = "Name"] name: String,
    #[description = "ISO3"] iso: String,
    #[description = "Flag"] emoji: String,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let mut r = ctx.data().lock().await;
    r.teams.push(Team {
        name,
        iso,
        flag: emoji,
    });
    ctx.reply("Succesful").await.unwrap();

    Ok(())
}

#[poise::command(slash_command, required_permissions = "ADMINISTRATOR")]
async fn add_game(
    ctx: PoiseContext<'_>,
    #[description = "Name"] name: String,
    #[description = "Kürzel"] short: String,
    #[description = "Team 1"] team1: String,
    #[description = "Team 2"] team2: String,
    #[description = "Anpfiff"] start_time: String,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
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

#[poise::command(slash_command, required_permissions = "SEND_MESSAGES")]
async fn list_games(ctx: PoiseContext<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
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
            ).ephemeral(true),
        )
        .await;

    if let Err(why) = r {
        println!("{why:?}");
    }

    Ok(())
}

#[poise::command(slash_command, required_permissions = "ADMINISTRATOR")]
async fn add_score(
    ctx: PoiseContext<'_>,
    #[description = "Das Kürzel des Spiels"] short: String,
    #[description = "Die Tore von Team 1"] team1_score: u16,
    #[description = "Die Tore von Team 2"] team2_score: u16,
    #[description = "Extra informationen zum Spiel"] msg: Option<String>,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let mut d = ctx.data().lock().await;
    let g = d
        .games
        .iter_mut()
        .find(|g| g.short == short)
        .ok_or("Kürzel gehört zu keinem Spiel")?;

    g.result = Some((team1_score, team2_score, msg.unwrap_or(String::new())));

    ctx.defer_ephemeral().await?;
    ctx.reply("Succesful").await?;

    Ok(())
}



async fn game_autocomplete<'a>(ctx: PoiseContext<'_>, partial: &'a str) -> impl Stream<Item = String> + 'a {
    let d = ctx.data.lock().await;
    let gs = d.games.clone();
    serenity::futures::stream::iter(gs).filter(move |n: &Game| serenity::futures::future::ready(n.short.starts_with(partial) && n.start_time > chrono::Local::now())).map(|g| format!("{} {} vs {} '{}'", g.name, g.team1_iso, g.team2_iso, g.short))
}

#[poise::command(slash_command, required_permissions = "SEND_MESSAGES")]
async fn bet(ctx: PoiseContext<'_>, #[description = "The game you want to set the bet for"] #[autocomplete = "game_autocomplete"] game: String, #[description = "Anzahl Tore Team 1"] #[min = 0] team1_score: u16, #[description = "Anzahl Tore Team 2"] #[min = 0] team2_score: u16) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let mut d = ctx.data().lock().await;

    let user = ctx.author().id;


    let game_tag = game.split("'").collect::<Vec<_>>().get(1).ok_or("Game could not be parsed!")?.to_string();
    let real_game = d.games.iter().find(|e| e.short == game_tag).ok_or("Game does not exist!")?.clone();

    let bets = d.bets.entry(real_game.short.clone()).or_insert(Vec::new());

    if let Some(bet) = bets.iter_mut().find(|b| b.user == user) {
        bet.team1 = team1_score;
        bet.team2 = team2_score;
    } else {
        bets.push(Bet {
            user,
            team1: team1_score,
            team2: team2_score
        });
    }

    ctx.reply(format!("Bet saved: {} {} vs {}  {}:{}", real_game.name, real_game.team1_iso, real_game.team2_iso, team1_score, team2_score)).await?;
    Ok(())
}