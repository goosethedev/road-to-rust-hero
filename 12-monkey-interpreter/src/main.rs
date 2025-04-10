mod lexer;

use std::io::{self, Write};

use lexer::Lexer;

const PROMPT: &str = ">> ";

fn main() {
    loop {
        // Print prompt
        print!("{}", PROMPT);
        io::stdout().flush().unwrap();

        // Read a line from input
        let mut line = String::new();
        let stdin = io::stdin();
        stdin
            .read_line(&mut line)
            .expect("Couldn't read from stdin");

        let mut lexer = Lexer::new(&line);
        let output = lexer.execute();
        dbg!(output);
    }
}
