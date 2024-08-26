use core::fmt;

#[derive(Clone, Copy, PartialEq)]
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
    pub fn new(value: u32, result: GuessResult) -> Self {
        Guess { value, result }
    }
}

impl fmt::Display for Guess {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = match self.result {
            GuessResult::Equal => "equal",
            GuessResult::Higher => "too high",
            GuessResult::Lower => "too low",
        };
        write!(f, "Number {} was {}", self.value, message)
    }
}
