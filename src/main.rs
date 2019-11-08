extern crate clap;
extern crate combat;

use std::error::Error;
use clap::clap_app;

fn main() -> Result<(), Box<dyn Error>> {

    // Define command line parameters
    let matches = clap_app!(combat =>
        (version: "1.0")
        (author: "Rafael Guillen <arthexis@gmail.com>")
        (about: "D&D Combat tools")
        (@arg ROSTER: -r --roster +takes_value "Roster definition file.")
        (@subcommand roll =>
            (about: "roll arbitrary formula")
            (@arg formula: "Formula to roll, example: d20+3"))
        (@subcommand init =>
            (about: "roll initiative")
            (@arg LAIR: -l --lair "Include lair actions at initiative 20."))
        (@subcommand join =>
            (about: "add a character to the roster")
            (@arg NAME: +required "Character name.")
            (@arg INIT: -i --init +takes_value "Set initiative formula.")
            (@arg ADV: -a --adv "Rolls initiative with advantage.")
            (@arg DIS: -d --dis "Rolls initiative with disadvantage.")
            (@arg HP: -h --hp +takes_value "Max HP value or formula."))
        (@subcommand kill =>
            (about: "remove a character from the roster")
            (@arg NAME: +required "Character name."))
        (@subcommand deal =>
            (about: "Deal damage to a character.")
            (@arg NAME: +required "Character name.")
            (@arg DMG: +required "Amount of damage."))
        (@subcommand heal =>
            (about: "Heal damage to a character.")
            (@arg NAME: +required "Character name.")
            (@arg DMG: +required "Amount of healing."))
    ).get_matches();

    // Load the party data from file
    let roster_file = matches.value_of("ROSTER").unwrap_or("roster.json");
    let mut roster = combat::Roster::load_from(roster_file);

    // Evaluate requested command
    match matches.subcommand() {
        ("roll", Some(m)) => { sc::roll(m); }
        ("init", Some(m)) => { sc::init(m, &roster); }
        ("join", Some(m)) => { sc::join(m, &mut roster); }
        ("kill", Some(m)) => { sc::kill(m, &mut roster); }
        ("deal", Some(m)) => { sc::deal(m, &mut roster); }
        ("heal", Some(m)) => { sc::heal(m, &mut roster); }
        _                 => { println!("Unrecognized command."); }
    }

    // Save changes to the roster
    // println!("Save roster: {:?}", roster);
    roster.save_to(roster_file);

    Ok(())
}

// Subcommands
pub mod sc {
    use clap::ArgMatches;

    // Sub-command: roll <formula>
    // Perform an arbitrary roll
    pub fn roll(matches: &ArgMatches) {
        if let Some(formula) = matches.value_of("formula") {
            println!("Roll {} = {}", formula, combat::Roll::from(&formula).roll());
        } else {
            eprintln!("Missing or invalid formula.");
        }
    }

    // Sub-command: init
    // Roll initiative for the entire party and encounter
    pub fn init(matches: &ArgMatches, roster: &combat::Roster) {
        let mut inits = roster.roll_inits();
        if inits.len() == 0 {
            println!("Roster is empty.");
            return
        }
        if matches.is_present("LAIR") {
            inits.push((20, String::from("LAIR ACTIONS")));
        }
        inits.sort_by(|a, b| b.0.cmp(&a.0));
        println!("Initiative rolls:");
        for init in inits.iter() {
            let head = format!("{}: {}", init.0, init.1);
            let tail = roster.get(&init.1).status();
            println!("{} {}", head, tail);
        }
    }

    // Sub-command: add <name> -i <formula>
    pub fn join(matches: &ArgMatches, roster: &mut combat::Roster) {

        // Name is always required
        let name = matches.value_of("NAME").unwrap();

        // Calculate initiative and initiative options
        let init = matches.value_of("INIT").unwrap_or("d20"); // Optional
        let mut init = combat::Roll::from(init);
        if matches.is_present("ADV") {
            init.with(combat::RollKind::Advantage);
        }
        if matches.is_present("DIS") {
            init.with(combat::RollKind::Disadvantage);
        }

        // Tell the user if we added or updated
        if roster.exists(name) {
            println!("Update {} in the roster:", name);
        } else {
            println!("Add {} to the roster.", name);
        }

        // Add character to the roster struct
        roster.join(name, init);

        // Asign a max HP if necessary
        if matches.is_present("HP") {
            let formula = matches.value_of("HP").unwrap();
            let ch = roster.get_mut(name);
            ch.hp.set_max(formula);
            println!("Set max HP to {} ({}).", formula, ch.hp.max());
        }
    }

    // Sub-command: kill <name>
    pub fn kill(matches: &ArgMatches, roster: &mut combat::Roster) {
        let name = matches.value_of("NAME").unwrap();  // Required
        if roster.exists(name) {
            roster.kill(name);
            println!("{} has been removed from the roster.", name);
        } else {
            println!("{} is not in the roster.", name);
        }
    }

    // Sub-command: deal <name> <dmg>
    pub fn deal(matches: &ArgMatches, roster: &mut combat::Roster) {
        let name = matches.value_of("NAME").unwrap();
        let dmg = combat::Roll::from(matches.value_of("DMG").unwrap()).roll();
        let ch = roster.get_mut(name);
        ch.hp.deal(dmg);
        println!("{} took {} damage, now has {} HP.", name, dmg, ch.hp.current());
    }

    // Sub-command: heal <name> <dmg>
    pub fn heal(matches: &ArgMatches, roster: &mut combat::Roster) {
        let name = matches.value_of("NAME").unwrap();
        let dmg = combat::Roll::from(matches.value_of("DMG").unwrap()).roll();
        let ch = roster.get_mut(name);
        ch.hp.heal(dmg);
        println!("{} healed {} damage, now has {} HP.", name, dmg, ch.hp.current());
    }

}
