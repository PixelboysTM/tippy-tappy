use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use poise::futures_util::lock::{Mutex, MutexGuard};
use serenity::all::{User, UserId};

pub struct Team {
    pub name: String,
    pub flag: String,
    pub iso: String,
}

#[derive(Clone)]
pub struct Game {
    pub name: String,
    pub short: String,
    pub team1_iso: String,
    pub team2_iso: String,
    pub start_time: chrono::DateTime<chrono::Local>,
    pub result: Option<(u16, u16, String)>,
}

pub struct DataInter {
    pub teams: Vec<Team>,
    pub games: Vec<Game>,
    pub bets: HashMap<String, Vec<Bet>>

}

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
    }
}

impl Data {
    pub async fn lock(&self) -> SaveGuard {
        let g = self.inter.lock().await;
        SaveGuard(g)
    }

    pub fn new() -> Self {
        Self {
            inter: Arc::new(Mutex::new(DataInter {
                games: vec![],
                teams: vec![],
                bets: HashMap::new()
            }))
        }
    }
}