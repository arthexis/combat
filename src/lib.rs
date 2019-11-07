extern crate serde;
extern crate serde_json;
extern crate dice_roller;

use serde::{Deserialize, Serialize};


/* Rolls
------------------------------------------------------------------------------------------- */

// Determine the kind of roll
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum RollKind {
    Normal,
    Advantage,
    Disadvantage
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
        self.kind = kind;
        self
    }

    // Execute the dice roll
    pub fn roll(&self) -> i64 {
        let dr = dice_roller::dice::Roller::parse(&self.formula);
        match self.kind {
            RollKind::Advantage => std::cmp::max(
                dr.roll().total(),  dr.roll().total()),
            RollKind::Disadvantage => std::cmp::min(
                dr.roll().total(), dr.roll().total()),
            RollKind::Normal => dr.roll().total()
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

// Represent a single PC or NPC
#[derive(Serialize, Deserialize, Debug)]
pub struct Character {
    name: String,
    kind: CharacterKind,
    init: Roll,
}

impl Character {

    // Roll initiative for this character
    pub fn roll_init(&self) -> i64 { self.init.roll() }
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

    pub fn join_pc(&mut self, name: &str, init: &str) {
        self.chars.insert(name.to_string(), Character {
            name: name.to_string(),
            init: Roll::from(init),
            kind: CharacterKind::PC
        });
    }

    pub fn exists(&self, name: &str) -> bool {
        self.chars.contains_key(name)
    }

    pub fn kill(&mut self, name: &str) {
        self.chars.remove(name);
    }

    // Roll initiative for every character in the roster
    pub fn roll_inits(&self) -> Vec<(i64, String)> {
        let mut inits = Vec::new();
        for (name, character) in self.chars.iter() {
            inits.push((character.roll_init(), name.clone()));
        }
        inits
    }

    // Save the roster to a file
    pub fn save_to(&self, file: &str) {
        std::fs::write(file, serde_json::to_string(self)
            .expect("Unable to write roster to file.")).unwrap();
    }
}
