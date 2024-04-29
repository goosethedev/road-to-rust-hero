use std::cmp::Ordering;
use std::fmt;

pub const MIN_GUESS: u32 = 1;
pub const MAX_GUESS: u32 = 100;

#[derive(Copy, Clone)]
pub enum GuessResult {
    Higher,
    Lower,
    Equal,
}

pub struct Guess {
    value: u32,
    result: GuessResult,
}

impl Guess {
    pub fn compare(guessed: u32, real: u32) -> Result<Self, String> {
        if guessed < MIN_GUESS || guessed > MAX_GUESS {
            return Err(format!(
                "Guessed value must be between {MIN_GUESS} and {MAX_GUESS}"
            ));
        }
        let result = match guessed.cmp(&real) {
            Ordering::Less => GuessResult::Lower,
            Ordering::Greater => GuessResult::Higher,
            Ordering::Equal => GuessResult::Equal,
        };

        Ok(Self {
            value: guessed,
            result,
        })
    }

    pub fn get_result(self: &Self) -> GuessResult {
        self.result
    }
}

impl fmt::Display for Guess {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = match self.result {
            GuessResult::Lower => "too low",
            GuessResult::Higher => "too high",
            _ => "is equal",
        };
        write!(f, "Number {} was {}!", self.value, message)
    }
}
