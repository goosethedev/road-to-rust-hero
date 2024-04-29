use rand::Rng;
use std::cmp::Ordering;
use std::io::{self, prelude::*}; // Necessary for pause

enum GuessResult {
    Higher,
    Lower,
}

struct Guess {
    value: u32,
    result: GuessResult,
}

// Prints a list of guesses, their details and total
fn print_guesses(guesses: &Vec<Guess>) {
    println!("Total guesses: {}", guesses.len());
    for guess in guesses {
        let value = guess.value;
        let message = match guess.result {
            GuessResult::Lower => "too low",
            GuessResult::Higher => "too high",
        };
        println!("- Number {value} was {message}!")
    }
    println!();
}

// Wait for any key to be pressed
// https://users.rust-lang.org/t/rusts-equivalent-of-cs-system-pause/4494/4
fn pause(message: &str) {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    // We want the cursor to stay at the end of the line,
    // so we print without a newline and flush manually.
    write!(stdout, "{message}").unwrap();
    stdout.flush().unwrap();

    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
}

fn main() {
    // Setup number to guess
    let secret_number = rand::thread_rng().gen_range(1..=100);
    // println!("DEBUG: Secret number: {secret_number}");

    // List of guesses
    let mut guesses: Vec<Guess> = Vec::new();

    loop {
        // Clear the screen
        print!("{}[2J", 27 as char);
        println!("Welcome to Guess the Number!\n");

        // Print guesses or introduction message
        if guesses.len() == 0 {
            println!("A random number between 1 and 100 has been generated.\n")
        } else {
            print_guesses(&guesses);
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
                pause("\nThat doesn't look like a valid number. Press any key to try again...");
                continue;
            }
        };

        // Verify guess is between 1-100
        if guess < 1 || guess > 100 {
            pause("\nThe number is between 1 and 100. Press any key to try again...");
            continue;
        }

        // Compare if smaller, bigger or correct
        match guess.cmp(&secret_number) {
            Ordering::Equal => {
                println!(
                    "You won in {} guesses! Thanks for playing!",
                    guesses.len() + 1
                );
                break;
            }
            diff => {
                guesses.push(Guess {
                    value: guess,
                    result: if diff == Ordering::Less {
                        GuessResult::Lower
                    } else {
                        GuessResult::Higher
                    },
                });
            }
        }
    }
}
