extern crate serde;
extern crate serde_json;
extern crate dice_roller;

use serde::{Deserialize, Serialize};


/* Rolls
------------------------------------------------------------------------------------------- */

// Determine the kind of roll
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub enum RollKind {
    Normal,
    Advantage,
    Disadvantage,
    Cancelled,
}

// Encapsulate a dice roll formula
// Takes advantage / disadvantage into consideration
#[derive(Serialize, Deserialize, Debug)]
pub struct Roll {
    formula: String,
    kind: RollKind,
}

impl Roll {

    // Create a new roll from a formula
    pub fn from(formula: &str) -> Roll {
        Roll {
            formula: formula.to_string(),
            kind: RollKind::Normal
        }
    }

    // Set advantage or disadvantage
    pub fn with(&mut self, kind: RollKind) -> &mut Roll {
        if self.kind == RollKind::Normal {
            self.kind = kind;
        } else if self.kind != kind {
            self.kind = RollKind::Cancelled;
        }
        self
    }

    // Execute the dice roll
    pub fn roll(&self) -> i64 {
        if !self.formula.contains("d") {
            self.formula.parse()
                .expect("Invalid format for formula.")
        } else {
            let dr = dice_roller::dice::Roller::parse(&self.formula);
            match self.kind {
                RollKind::Advantage    =>
                    std::cmp::max(dr.roll().total(),  dr.roll().total()),
                RollKind::Disadvantage =>
                    std::cmp::min(dr.roll().total(), dr.roll().total()),
                _                      =>
                    dr.roll().total()
            }
        }
    }

    // Use this roll to make a check
    pub fn check(&self, dc: i64) -> bool { self.roll() >= dc }

}


/* Characters
------------------------------------------------------------------------------------------- */

#[derive(Serialize, Deserialize, Debug)]
pub enum CharacterKind {
    PC,
    NPC
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct HitPoints {
    max: i64,
    current: i64,
    temp: i64,
}

impl HitPoints {

    pub fn from(formula: &str) -> HitPoints {
        let total = Roll::from(formula).roll();
        HitPoints { max: total, current: total, temp: 0 }
    }

    // Change the max hitpoints
    pub fn set_max(&mut self, formula: &str) {
        let total = Roll::from(formula).roll();
        self.current = self.current + (total - self.max);
        self.max = total;
    }

    // Return true if max hit points are set
    pub fn is_set(&self) -> bool {
        self.max > 0
    }

    // Set temporary hitpoints
    pub fn temp(&mut self, hp: i64) {
        if hp > self.temp { self.temp = hp; }
    }

    // Deal damage
    pub fn deal(&mut self, dmg: i64) {
        let mut _dmg = dmg;
        if self.temp > 0 {
            self.temp = std::cmp::max(self.temp - _dmg, 0);
            _dmg = _dmg - self.temp;
        }
        if _dmg > 0 {
            self.current = std::cmp::max(self.current - _dmg, 0)
        }
    }

    // Heal damage
    pub fn heal(&mut self, dmg: i64) {
        self.current = std::cmp::min(self.current + dmg, self.max)
    }

    // Get current hit-points
    pub fn current(&self) -> i64 { self.current }

    // Get max hit-points
    pub fn max(&self) -> i64 { self.max }

}


// Represent a single PC or NPC
#[derive(Serialize, Deserialize, Debug)]
pub struct Character {
    name: String,
    pub init: Roll,
    pub hp: HitPoints,
}

impl Character {

    // Return true if the character is dead
    pub fn dead(&self) -> bool {
        self.hp.max > 0 && self.hp.current < 1
    }

    // Return an user friendly status message
    pub fn status(&self) -> String {
        if self.dead() {
            "DEAD".to_string()
        } else if self.hp.is_set() {
            format!("{} HP", self.hp.current())
        } else {
            "".to_string()
        }
    }
}


/* Roster
------------------------------------------------------------------------------------------- */

// A list of characters that roll initiative together
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Roster {
    chars: std::collections::HashMap<String, Character>,
}

impl Roster {

    // Load a new roster from a file
    // If the file cannot be loaded, return an empty roster
    pub fn load_from(file: &str) -> Roster {
        if let Ok(data) = std::fs::read_to_string(file) {
            println!("Using roster data from file {}", file);
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            println!("File {} could not be read, create blank roster.", file);
            Roster::default()
        }
    }

    // Save the roster to a file
    pub fn save_to(&self, file: &str) {
        std::fs::write(file, serde_json::to_string(self)
            .expect("Unable to write roster to file.")).unwrap();
    }

    // Add a new character to the roster
    pub fn join(&mut self, name: &str, init: Roll) {
        self.chars.insert(name.to_string(), Character {
            name: name.to_string(),
            hp: HitPoints::default(),
            init,
        });
    }

    // Check if a character exists
    pub fn exists(&self, name: &str) -> bool {
        self.chars.contains_key(name)
    }

    // Remove a character from the roster
    pub fn kill(&mut self, name: &str) {
        self.chars.remove(name);
    }

    // Retrieve a specific caracter
    pub fn get(&self, name: &str) -> &Character {
        self.chars.get(name).expect("No character found with that name.")
    }

    // Retrieve a specific caracter mutably
    pub fn get_mut(&mut self, name: &str) -> &mut Character {
        self.chars.get_mut(name).expect("No character found with that name.")
    }

    // Roll initiative for every character in the roster
    pub fn roll_inits(&self) -> Vec<(i64, String)> {
        let mut inits = Vec::new();
        for (name, character) in self.chars.iter() {
            inits.push((character.init.roll(), name.clone()));
        }
        inits
    }

    // Remove dead characters
    pub fn wipe(&mut self) {
        // Get the names of all dead characters
        let dead: Vec<_> = self.chars.iter()
            .filter(|(_, ch)| ch.dead())
            .map(|(name, _)| name.clone())
            .collect();
        // Remove the characters by name
        for name in dead {
            println!("{} is dead, removing.", name);
            self.chars.remove(&name);
        }
    }

}
