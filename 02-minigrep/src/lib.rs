use std::{env, error::Error, fs};

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

pub struct Config {
    query: String,
    file_path: String,
    ignore_case: bool,
}

impl Config {
    pub fn build(args: &[String]) -> Result<Self, &'static str> {
        if args.len() < 3 {
            return Err("not enough arguments");
        }

        let query = args[1].clone();
        let file_path = args[2].clone();
        let ignore_case = env::var("IGNORE_CASE").is_ok();

        Ok(Self {
            query,
            file_path,
            ignore_case,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_result() {
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
    fn multiple_results() {
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
    fn case_sensitive() {
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
    fn case_insensitive() {
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
