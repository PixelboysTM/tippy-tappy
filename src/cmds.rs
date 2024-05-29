use std::collections::HashMap;
use std::fmt::Display;
use ascii_table::{Align, AsciiTable};
use itertools::Itertools;
use poise::futures_util::StreamExt;
use poise::{Command, CreateReply};
use serenity::all::{Channel, Colour, CreateEmbed, CreateEmbedAuthor, CreateMessage, User};
use serenity::futures::Stream;

use crate::data::{Bet, Data, Game, Team};
use crate::{Error, POINTS_CORRECT, POINTS_TEAM, POINTS_TENDENZ, PoiseContext};

pub fn get_cmds() -> Vec<Command<Data, Error>> {
    vec![
        age(),
        get_teams(),
        add_team(),
        add_game(),
        list_games(),
        add_score(),
        bet(),
        get_bets(),
        print_overview(),
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
            CreateReply::default()
                .reply(true)
                .embed(
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
                )
                .ephemeral(true),
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
        start_time: t,
    });

    ctx.reply("Succesful").await.unwrap();

    Ok(())
}

#[poise::command(slash_command, required_permissions = "SEND_MESSAGES")]
async fn list_games(ctx: PoiseContext<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let r = ctx
        .send(
            CreateReply::default()
                .reply(true)
                .embed(
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
                )
                .ephemeral(true),
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

async fn game_autocomplete<'a>(
    ctx: PoiseContext<'_>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    let d = ctx.data.lock().await;
    let gs = d.games.clone();
    serenity::futures::stream::iter(gs)
        .filter(move |n: &Game| {
            serenity::futures::future::ready(
                n.short.starts_with(partial) && n.start_time > chrono::Local::now().naive_local(),
            )
        })
        .map(|g| {
            format!(
                "{} {} vs {} '{}'",
                g.name, g.team1_iso, g.team2_iso, g.short
            )
        })
}

#[poise::command(slash_command, required_permissions = "SEND_MESSAGES")]
async fn bet(
    ctx: PoiseContext<'_>,
    #[description = "The game you want to set the bet for"]
    #[autocomplete = "game_autocomplete"]
    game: String,
    #[description = "Anzahl Tore Team 1"]
    #[min = 0]
    team1_score: u16,
    #[description = "Anzahl Tore Team 2"]
    #[min = 0]
    team2_score: u16,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let mut d = ctx.data().lock().await;

    let user = ctx.author().id;

    let game_tag = game
        .split("'")
        .collect::<Vec<_>>()
        .get(1)
        .ok_or("Game could not be parsed!")?
        .to_string();
    let real_game = d
        .games
        .iter()
        .find(|e| e.short == game_tag)
        .ok_or("Game does not exist!")?
        .clone();

    if real_game.start_time <= chrono::Local::now().naive_local() {
        ctx.reply("Der Tipp für dieses Spiel kann nicht mehr verändert werden!")
            .await?;
        return Ok(());
    }

    let bets = d.bets.entry(real_game.short.clone()).or_insert(Vec::new());

    if let Some(bet) = bets.iter_mut().find(|b| b.user == user) {
        bet.team1 = team1_score;
        bet.team2 = team2_score;
    } else {
        bets.push(Bet {
            user,
            team1: team1_score,
            team2: team2_score,
        });
    }

    ctx.reply(format!(
        "Bet saved: {} {} vs {}  {}:{}",
        real_game.name, real_game.team1_iso, real_game.team2_iso, team1_score, team2_score
    ))
    .await?;
    Ok(())
}

#[poise::command(slash_command, required_permissions = "SEND_MESSAGES")]
async fn get_bets(ctx: PoiseContext<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let d = ctx.data().lock().await;
    let user = ctx.author().id;
    let bets = d
        .bets
        .iter()
        .filter_map(|(k, v)| {
            v.iter()
                .find(|b| b.user == user)
                .map(|b| (k.clone(), b.clone()))
        })
        .collect::<Vec<_>>();

    let r = ctx
        .send(
            CreateReply::default()
                .reply(true)
                .embed(
                    CreateEmbed::new()
                        .fields(bets.iter().map(|(short, b)| {
                            let g = d.games.iter().find(|g| &g.short == short).unwrap();
                            (
                                format!("{}", g.name),
                                format!(
                                    "({}) {} : {} ({})",
                                    g.team1_iso, b.team1, b.team2, g.team2_iso
                                ),
                                true,
                            )
                        }))
                        .author(CreateEmbedAuthor::new("Tippy Tappy"))
                        .title("Bets")
                        .description("All your bets")
                        .color(Colour::DARK_ORANGE),
                )
                .ephemeral(true),
        )
        .await;

    Ok(())
}

#[poise::command(slash_command, required_permissions = "ADMINISTRATOR")]
async fn print_overview(ctx: PoiseContext<'_>, channel: Option<Channel>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let channel = channel.map(|c| c.id()).unwrap_or_else(|| ctx.channel_id());

    let d = ctx.data().lock().await;

    let mut user_bets = HashMap::new();
    for (game_short, bets) in d.bets.iter() {
        let g = d.games.iter().find(|g| &g.short == game_short).unwrap();
        let result = if let Some(r) = g.result.as_ref() {
            r
        } else {
            for bet in bets {
                user_bets.entry(bet.user).or_insert(0u32);
            }

            continue;
        };

        for bet in bets {
            let points = if bet.team1 == result.0 && bet.team2 == result.1 {
                POINTS_CORRECT
            } else if (bet.team1 as i16 - bet.team2 as i16) == (result.0 as i16 - result.1 as i16) {
                POINTS_TENDENZ
            } else if (bet.team1 > bet.team2 && result.0 > result.1) || (bet.team1 < bet.team2 && result.0 < result.1) {
                POINTS_TEAM
            } else { 0 };

            let mut ps = user_bets.entry(bet.user).or_insert(0u32);
            *ps +=  points;
        }
    }

    let mut points_table = AsciiTable::default();
    // points_table.set_max_width(70);
    points_table.column(0).set_header("Username");
    points_table.column(1).set_header("Points");

    let mut points_table_data: Vec<Vec<String>> = Vec::new();
    for (k, v) in user_bets {
        let name = k.to_user(ctx.http()).await.map(|u| u.name.clone()).unwrap_or("UNKNOWN".to_string());
        points_table_data.push(vec![name, v.to_string()]);
    }

    let points_table_string = points_table.format(points_table_data);


    channel
        .send_message(&ctx, CreateMessage::new().content(format!("# Übersicht\n```\n{points_table_string}\n```")))
        .await?;

    let mut games_table = AsciiTable::default();
    // games_table.set_max_width(70);
    games_table.column(0).set_header("Spiel");
    games_table.column(1).set_header("Kontrahent 1");
    games_table.column(2).set_header("vs");
    games_table.column(3).set_header("Kontrahent 2");
    games_table.column(4).set_header("Anpfiff");
    games_table.column(5).set_header("Ergebnis");

    let mut games = d.games.iter().cloned().collect::<Vec<_>>();

    games.sort_by(|g1,g2| g1.start_time.cmp(&g2.start_time));

    let mut games_table_data: Vec<Vec<String>> = Vec::new();
    for game in games {
        let t1 = d.teams.iter().find(|t| t.iso == game.team1_iso).unwrap();
        let t2 = d.teams.iter().find(|t| t.iso == game.team2_iso).unwrap();
        let r = game.result.map(|r| format!("{}:{} {}", r.0, r.1, r.2)).unwrap_or("-:-".to_string());
        games_table_data.push(vec![
           game.name,
           t1.name.clone(),
           "vs".to_string(),
           t2.name.clone(),
           game.start_time.format("%d.%m.%Y %H:%M Uhr").to_string(),
           r
        ]);
    }

    let games_table_string = games_table.format(games_table_data);
    channel
        .send_message(&ctx, CreateMessage::new().content(format!("# Spiele\n```\n{games_table_string}\n```")))
        .await?;

    let mut games = d.games.iter().filter(|g| g.start_time <= chrono::Local::now().naive_local() ).cloned().collect::<Vec<_>>();

    games.sort_by(|g1,g2| g1.start_time.cmp(&g2.start_time));

    let users = d.bets.iter().map(|(k,v)| v.iter().map(|b| b.user).collect::<Vec<_>>()).flatten().unique().collect::<Vec<_>>();

    let mut tipps_table = AsciiTable::default();
    tipps_table.column(0).set_header("Spieler\\Game");
    for (i, g) in games.iter().enumerate() {
        tipps_table.column(i+1).set_header(g.short.clone());
    }

    let mut tipps_table_data: Vec<Vec<String>> = Vec::new();

    for user in users {
        let u = user.to_user(ctx.http()).await;
        let mut cols = Vec::new();
        cols.push(u.map(|u| u.name).unwrap_or("UNKNOWN".to_string()));
        for g in &games {
            let bets = d.bets.get(&g.short);
            let tip = if let Some(bets) = bets {
                let bet = bets.iter().find(|b| b.user == user);
                bet.map(|b| format!("{}:{}", b.team1, b.team2)).unwrap_or("-:-".to_string())
            } else {
                "-:-".to_string()
            };
            cols.push(tip);
        }

        tipps_table_data.push(cols);
    }

    let tipps_table_string = tipps_table.format(tipps_table_data);
    channel
        .send_message(&ctx, CreateMessage::new().content(format!("# Tipps\n```\n{tipps_table_string}\n```")))
        .await?;

    ctx.reply("Send Msg to channel").await?;

    // let mut ascii_table = AsciiTable::default();
    // ascii_table.set_max_width(70);
    // ascii_table.column(0).set_header("H1").set_align(Align::Left);
    // ascii_table.column(1).set_header("H2").set_align(Align::Center);
    // ascii_table.column(2).set_header("H3").set_align(Align::Right);
    //
    // let data: Vec<Vec<&dyn Display>> = vec![vec![&'v', &'v', &'v'], vec![&123, &456, &"dsadssad"]];
    //
    // let out = ascii_table.format(data);
    //
    // channel
    //     .send_message(&ctx, CreateMessage::new().content(format!("```\n{out}\n```")))
    //     .await?;

    Ok(())
}
