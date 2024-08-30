use std::{env, error::Error, fs, process};

fn main() -> Result<(), Box<dyn Error>> {
    // Parse arguments from command line as Config obj
    let config = minigrep::Config::build(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    let contents = fs::read_to_string(&config.file_path)?;
    let results = minigrep::search(&config.query, &contents, config.ignore_case);
    println!("{}", results.join("\n"));
    Ok(())
}
