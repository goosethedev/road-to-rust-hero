use crate::guess::{Guess, GuessResult, MAX_GUESS, MIN_GUESS};
use rand::Rng;
use std::io;

mod guess;
mod helpers;

fn main() {
    // Setup number to guess
    let secret_number = rand::thread_rng().gen_range(MIN_GUESS..=MAX_GUESS);
    // println!("DEBUG: Secret number: {secret_number}");

    // List of guesses
    let mut guesses: Vec<Guess> = Vec::new();

    loop {
        // Clear the screen
        print!("{}[2J", 27 as char);
        println!("Welcome to Guess the Number!\n");

        // Print guesses or introduction message
        if guesses.len() == 0 {
            println!("A random number between {MIN_GUESS} and {MAX_GUESS} has been generated.\n")
        } else {
            helpers::print_guesses(&guesses);
        }

        // Read from user input
        println!("Your guess (or 'q' to exit): ");
        let mut guess = String::new();
        io::stdin()
            .read_line(&mut guess)
            .expect("Failed to read input");

        // Quit if "q"
        // Important to trim! stdin also takes the \n character at the end
        if guess.trim().eq("q") {
            println!("\nBetter luck next time! The number was {secret_number}");
            break;
        }

        // Parse to number
        let guess: u32 = match guess.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                helpers::pause(
                    "\nThat doesn't look like a valid number. Press any key to try again...",
                );
                continue;
            }
        };

        // Create guess and compare
        match Guess::compare(guess, secret_number) {
            Ok(g) => match g.get_result() {
                // Player won! Exit the loop
                GuessResult::Equal => {
                    println!(
                        "You won in {} guesses! Thanks for playing!",
                        guesses.len() + 1
                    );
                    break;
                }
                // Add to guesses and keep going
                _ => guesses.push(g),
            },
            // Error when mapping the Guess
            Err(error) => helpers::pause(&error),
        }
    }
}
