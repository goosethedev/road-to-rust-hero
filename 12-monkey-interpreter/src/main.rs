mod lexer;
mod parser;

use std::io::Write;

use lexer::Lexer;
use parser::Ast;

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
        let ast = Ast::new(lexer).parse();
        dbg!(ast);
    }
}
