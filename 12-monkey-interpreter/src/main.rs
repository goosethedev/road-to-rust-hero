mod lexer;

use std::io::Write;

use lexer::Lexer;

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

        let lexer = Lexer::new(&line);
        let output: Vec<_> = lexer.into_iter().collect();
        dbg!(output);
    }
}
