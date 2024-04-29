use crate::guess;
use std::io::{self, prelude::*}; // Necessary for pause

// Prints a list of guesses, their details and total
pub fn print_guesses(guesses: &Vec<guess::Guess>) {
    let results_block = guesses
        .iter()
        .map(|g| format!("- {g}"))
        .collect::<Vec<String>>()
        .join("\n");
    print!("Total guesses: {}\n{}\n\n", guesses.len(), results_block)
}

// Wait for any key to be pressed
// https://users.rust-lang.org/t/rusts-equivalent-of-cs-system-pause/4494/4
pub fn pause(message: &str) {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    // We want the cursor to stay at the end of the line,
    // so we print without a newline and flush manually.
    write!(stdout, "{message}").unwrap();
    stdout.flush().unwrap();

    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
}
