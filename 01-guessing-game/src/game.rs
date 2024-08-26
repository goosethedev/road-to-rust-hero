pub use crate::guess::{Guess, GuessResult};
use rand::Rng;
use std::cmp::Ordering;

pub struct GuessingGame {
    min_value: u32,
    max_value: u32,
    secret_number: u32,
    guesses: Box<Vec<Guess>>,
}

impl GuessingGame {
    /// Create a new guessing game
    pub fn new(min_value: u32, max_value: u32) -> Self {
        let secret_number = rand::thread_rng().gen_range(min_value..=max_value);
        Self {
            min_value,
            max_value,
            secret_number,
            guesses: Box::new(vec![]),
        }
    }

    /// Perform a guess
    pub fn make_guess(&mut self, guess: u32) -> Result<GuessResult, String> {
        // Return if out of range
        if guess < self.min_value || guess > self.max_value {
            return Err("Value out of range".to_string());
        }

        // Perform and return comparison
        let result = match guess.cmp(&self.secret_number) {
            Ordering::Less => GuessResult::Lower,
            Ordering::Greater => GuessResult::Higher,
            Ordering::Equal => GuessResult::Equal,
        };

        // Add to guesses and return
        self.guesses.push(Guess::new(guess, result));
        Ok(result)
    }

    pub fn get_guess_amount(&self) -> usize {
        self.guesses.len()
    }

    pub fn get_secret_number(&self) -> u32 {
        self.secret_number
    }

    /// Print a prettified list of guesses
    pub fn print_guesses(&self) {
        let results_block = self
            .guesses
            .iter()
            .map(|g| format!("- {g}"))
            .collect::<Vec<String>>()
            .join("\n");
        print!(
            "Total guesses: {}\n{}\n\n",
            self.get_guess_amount(),
            results_block
        )
    }
}
