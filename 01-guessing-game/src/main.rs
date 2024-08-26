use guessing_game::{
    game::{GuessResult, GuessingGame},
    helpers,
};
use std::io;

pub const MIN_GUESS: u32 = 1;
pub const MAX_GUESS: u32 = 100;

fn main() {
    // Create game object
    let mut game = GuessingGame::new(MIN_GUESS, MAX_GUESS);

    loop {
        // Clear the screen
        print!("{}[2J", 27 as char);
        println!("Welcome to Guess the Number!\n");

        // Print list of last guesses or introduction message
        let guess_amount = game.get_guess_amount();
        if guess_amount == 0 {
            println!("A random number between {MIN_GUESS} and {MAX_GUESS} has been generated.\n")
        } else {
            game.print_guesses();
        }

        // Read from user input
        println!("Your guess (or 'q' to exit):");
        let mut guess = String::new();
        io::stdin()
            .read_line(&mut guess)
            .expect("Failed to read input");

        // Quit if "q"
        // Important to trim! stdin also takes the \n character at the end
        if guess.trim().eq("q") {
            println!(
                "\nBetter luck next time! The number was {}",
                game.get_secret_number()
            );
            break;
        }

        // Parse to number
        let Ok(guess) = guess.trim().parse::<u32>() else {
            helpers::pause(
                "\nThat doesn't look like a valid number. Press any key to try again...",
            );
            continue;
        };

        // Make the guess and check the result
        let Ok(result) = game.make_guess(guess) else {
            helpers::pause(
                &format!("\nNumber out of range. It should be between {MIN_GUESS} and {MAX_GUESS}. Press any key to try again..."),
            );
            continue;
        };

        if result == GuessResult::Equal {
            println!(
                "\nYou won using {} guesses! Thanks for playing!",
                guess_amount + 1
            );
            break;
        }
    }
}
