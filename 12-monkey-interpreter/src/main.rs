mod evaluate;
mod lexing;
mod parsing;

use std::io::Write;

use crate::evaluate::eval;
use crate::lexing::Lexer;
use crate::parsing::Parser;

const PROMPT: &str = ">> ";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut global_scope = std::collections::HashMap::new();
    loop {
        // Print prompt
        print!("{}", PROMPT);
        std::io::stdout().flush()?;

        // Read a line from input
        let mut line = String::new();
        let stdin = std::io::stdin();
        stdin.read_line(&mut line)?;

        // Tokenize and build AST
        let lexer = Lexer::new(&line);
        let ast = match Parser::new(lexer).parse() {
            Ok(ast) => ast,
            Err(errors) => {
                errors.iter().for_each(|e| eprintln!("{e}"));
                continue;
            }
        };

        // Eval and print results
        match eval(ast, &mut global_scope) {
            Ok(obj) => println!("{obj}"),
            Err(e) => eprintln!("{e}"),
        }
    }
}
