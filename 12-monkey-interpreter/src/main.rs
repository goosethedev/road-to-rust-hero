mod lexing;
mod parsing;

use std::io::Write;

use lexing::Lexer;
use parsing::Parser;

const PROMPT: &str = ">> ";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    loop {
        // Print prompt
        print!("{}", PROMPT);
        std::io::stdout().flush()?;

        // Read a line from input
        let mut line = String::new();
        let stdin = std::io::stdin();
        stdin.read_line(&mut line)?;

        // Process
        let lexer = Lexer::new(&line);
        let ast: Vec<_> = Parser::new(lexer).collect();

        // Print results
        for res in ast {
            match res {
                Ok(s) => println!("{s}"),
                Err(e) => eprintln!("{e}"),
            }
        }
    }
}
