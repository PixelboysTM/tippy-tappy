use std::collections::HashMap;
use std::env;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use chrono::Utc;

use poise::futures_util::lock::{Mutex, MutexGuard};
use serde::{Deserialize, Serialize};
use serenity::all::UserId;
use serenity::model::Timestamp;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Team {
    pub name: String,
    pub flag: String,
    pub iso: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Game {
    pub name: String,
    pub short: String,
    pub team1_iso: String,
    pub team2_iso: String,
    pub start_time: chrono::NaiveDateTime,
    pub result: Option<(u16, u16, String)>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataInter {
    pub teams: Vec<Team>,
    pub games: Vec<Game>,
    pub bets: HashMap<String, Vec<Bet>>,
    pub global_bets: HashMap<String, GlobalBet>
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Hash, Clone)]
pub struct GlobalBet {
    pub name: String,
    pub short: String,
    pub points: u16,
    pub start_time: chrono::NaiveDateTime,
    pub result: Option<String>,
    pub bets: Vec<(UserId, String)>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Bet {
    pub user: UserId,
    pub team1: u16,
    pub team2: u16
}

pub struct Data {
    inter: Arc<Mutex<DataInter>>,
}

pub struct SaveGuard<'a>(MutexGuard<'a, DataInter>);

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
        let d: &DataInter = &self;
        let dat = serde_json::to_string(d).unwrap();
        if let Ok(f) = env::var("SAVE_FILE") {
            std::fs::write(f, dat).unwrap();
        }
    }
}

impl Data {
    pub async fn lock(&self) -> SaveGuard {
        let g = self.inter.lock().await;
        SaveGuard(g)
    }

    pub fn new() -> Self {
        Self {
            inter: Arc::new(Mutex::new(load_data()))
        }
    }
}

fn load_data() -> DataInter {
    let p = env::var("SAVE_FILE").unwrap_or("".to_string());

    if p.is_empty() {
        return DataInter {
            games: vec![],
            teams: vec![],
            bets: HashMap::new(),
            global_bets: HashMap::new()
        }
    }

    let f = std::fs::read_to_string(p);
    if let Ok(f) = f {
        serde_json::from_str(&f).unwrap()
    } else {
        println!("Loading file failed");
        DataInter {
            games: vec![],
            teams: vec![],
            bets: HashMap::new(),
            global_bets: HashMap::new()
        }
    }
}