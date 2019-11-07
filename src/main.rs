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
            (@arg ADV: -a --adv "Rolls initiative with advantage."))
        (@subcommand kill =>
            (about: "remove a character from the roster")
            (@arg NAME: +required "Character name."))
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
        _                 => { println!("Unrecognized command."); }
    }

    // Save changes to the roster
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
            println!("{}: {}", init.0, init.1);
        }
    }

    // Sub-command: add <character> -i <formula>
    pub fn join(matches: &ArgMatches, roster: &mut combat::Roster) {
        let name = matches.value_of("NAME").unwrap();  // Required
        let init = matches.value_of("INIT").unwrap_or("d20"); // Optional
        roster.join_pc(name, init);
        println!("{} has been added to the roster.", name);
    }

    // Sub-command: kill <character>
    pub fn kill(matches: &ArgMatches, roster: &mut combat::Roster) {
        let name = matches.value_of("NAME").unwrap();  // Required
        if roster.exists(name) {
            roster.kill(name);
            println!("{} has been removed from the roster.", name);
        } else {
            println!("{} is not in the roster.", name);
        }
    }

}
