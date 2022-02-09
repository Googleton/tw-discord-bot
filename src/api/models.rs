use std::collections::HashMap;
use std::error::Error;

use std::path::Path;
use urlencoding::decode;
use serde::{Deserialize};
use tracing::{info, debug};

#[derive(Debug, Deserialize)]
pub struct Village {
    pub id: u32,
    pub name: String,
    pub x: u32,
    pub y: u32,
    pub player_id: u32,
    pub rank: u32
}

#[derive(Debug, Deserialize)]
pub struct Player {
    pub id: u32,
    pub name: String,
    pub tribe_id: u32,
    pub villages: u32,
    pub points: u32,
    pub rank: u32
}

#[derive(Debug, Deserialize)]
pub struct Tribe {
    pub id: u32,
    pub name: String,
    pub tag: String,
    pub members: u32,
    pub villages: u32,
    pub points: u32,
    pub all_points: u32,
    pub rank: u32
}

pub struct TribalWars {
    pub villages: HashMap<u32, Village>,
    pub players: HashMap<u32, Player>,
    pub tribes: HashMap<u32, Tribe>
}

impl TribalWars {
    pub fn load() -> TribalWars  {
        debug!("Loading TribalWars data");

        let mut tw = TribalWars {
            tribes: HashMap::new(),
            players: HashMap::new(),
            villages: HashMap::new()
        };

        match tw.load_tribes() {
            Ok(_) => {}
            Err(err) => { panic!("Could not load tribes! {:?}", err)}
        };

        match tw.load_players() {
            Ok(_) => {}
            Err(err) => { panic!("Could not load players! {:?}", err)}
        };

        match tw.load_villages() {
            Ok(_) => {}
            Err(err) => { panic!("Could not load villages! {:?}", err)}
        };

        tw
    }

    fn load_tribes(&mut self) -> Result<(), Box<dyn Error>>{
        debug!("Loading TribalWars tribes");
        let mut rdr = csv::ReaderBuilder::new().has_headers(false).from_path(Path::new("ally.txt"))?;
        for result in rdr.deserialize() {
            let mut tribe: Tribe = result?;
            tribe.name = decode(tribe.name.replace("+", " ").as_str())?.into_owned();
            self.tribes.insert(tribe.id, tribe);
        }

        info!("Loaded {} tribes.", self.tribes.len());

        Ok(())
    }

    fn load_players(&mut self) -> Result<(), Box<dyn Error>>{
        debug!("Loading TribalWars players");
        let mut rdr = csv::ReaderBuilder::new().has_headers(false).from_path(Path::new("player.txt"))?;
        for result in rdr.deserialize() {
            let mut player: Player = result?;
            player.name = decode(player.name.replace("+", " ").as_str())?.into_owned();
            self.players.insert(player.id, player);
        }

        info!("Loaded {} players.", self.players.len());

        Ok(())
    }

    fn load_villages(&mut self) -> Result<(), Box<dyn Error>>{
        debug!("Loading TribalWars villages");
        let mut rdr = csv::ReaderBuilder::new().has_headers(false).from_path(Path::new("village.txt"))?;
        for result in rdr.deserialize() {
            let mut village: Village = result?;
            village.name = decode(village.name.replace("+", " ").as_str())?.into_owned();
            self.villages.insert(village.id, village);
        }

        info!("Loaded {} villages.", self.villages.len());

        Ok(())
    }

    pub fn tribe(&self, id: u32) -> Option<&Tribe> {
        for tribe in self.tribes.values() {
            if tribe.id == id {
                return Some(tribe);
            }
        }

        return None;
    }

    pub fn tribe_by_name(&self, name: &str) -> Option<&Tribe> {
        for tribe in self.tribes.values() {
            if tribe.name.eq_ignore_ascii_case(name) || tribe.tag.eq_ignore_ascii_case(name) {
                return Some(tribe);
            }
        }

        return None;
    }

    pub fn player(&self, id: u32) -> Option<&Player> {
        for player in self.players.values() {
            if player.id == id {
                return Some(player);
            }
        }

        return None;
    }

    pub fn player_by_name(&self, name: &str) -> Option<&Player> {
        for player in self.players.values() {
            if player.name.eq_ignore_ascii_case(name) {
                return Some(player);
            }
        }

        return None;
    }

    pub fn players_by_tribe(&self, name: &str) -> Option<Vec<&Player>> {
        let tribe = self.tribe_by_name(name);

        if tribe.is_none() {
            return None;
        }

        let tribe = tribe.unwrap();

        let mut result = self.players.values().into_iter().filter(|v| v.tribe_id == tribe.id).collect();

        Some(result)
    }

    pub fn village(&self, id: &u32) -> Option<&Village> {
        for village in self.villages.values() {
            if village.id == *id {
                return Some(village);
            }
        }

        return None;
    }

    pub fn village_at(&self, x: &u32, y: &u32) -> Option<&Village> {
        for village in self.villages.values() {
            if village.x == *x && village.y == *y {
                return Some(village);
            }
        }

        return None;
    }

    pub fn villages_for(&self, name: &str) -> Option<Vec<&Village>> {
        let player = self.player_by_name(name);

        if player.is_none() {
            return None;
        }

        let player = player.unwrap();

        let mut result = self.villages.values().into_iter().filter(|v| v.player_id == player.id).collect();

        Some(result)
    }
}