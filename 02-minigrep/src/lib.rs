use config::Config;
use std::{error::Error, fs};

pub mod config;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(&config.file_path)?;

    let results = search(&config.query, &contents, config.ignore_case);

    println!("{}", results.join("\n"));
    Ok(())
}

fn turn_to_lowercase(s: &str, case: bool) -> String {
    if case {
        s.to_lowercase()
    } else {
        s.to_string()
    }
}

fn search<'a>(query: &str, contents: &'a str, icase: bool) -> Vec<&'a str> {
    let query: String = turn_to_lowercase(query, icase);
    contents
        .lines()
        .filter(|line| turn_to_lowercase(line, icase).contains(&query))
        .collect::<Vec<&str>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_gives_one_result() {
        let query = "duct";
        let contents = "\
rust:
safe, fast, productive.
pick three.";

        assert_eq!(
            vec!["safe, fast, productive."],
            search(query, contents, false)
        );
    }

    #[test]
    fn search_gives_multiple_results() {
        let query = "st";
        let contents = "\
rust:
safe, fast, productive.
pick three.";

        assert_eq!(
            vec!["rust:", "safe, fast, productive."],
            search(query, contents, false)
        );
    }

    #[test]
    fn search_with_case_sensitive() {
        let query = "HARD";
        let contents = "\
Rust is HaRd
but not thaaat HARD is it?";

        assert_eq!(
            vec!["but not thaaat HARD is it?"],
            search(query, contents, false)
        );
    }

    #[test]
    fn search_with_case_insensitive() {
        let query = "harD";
        let contents = "\
Rust is HaRd
but not thaaat HARD is it?";

        assert_eq!(
            vec!["Rust is HaRd", "but not thaaat HARD is it?"],
            search(query, contents, true)
        );
    }
}
