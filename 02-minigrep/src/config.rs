use regex::Regex;
use std::{cmp::Ordering, env};

const OPT_IGNORE_CASE: &str = "IGNORE_CASE";

#[derive(PartialEq, Debug)]
pub struct Config {
    pub query: String,
    pub file_path: String,
    pub ignore_case: bool,
}

impl Config {
    fn map_option(&mut self, option: String) -> Result<(), String> {
        match option.as_str() {
            "-i" | "--case-insensitive" => {
                self.ignore_case = true;
                Ok(())
            }
            // add more in the future
            opt => Err(format!("Unknown option: {opt}")),
        }
    }

    pub fn build(mut env_args: impl Iterator<Item = String>) -> Result<Self, String> {
        // Discard the command (first) arg
        env_args.next().unwrap();

        let mut args = vec![];
        let mut opts = vec![];

        let opts_regex = Regex::new("^-.+").unwrap();
        for arg in env_args {
            if opts_regex.is_match(&arg) {
                opts.push(arg);
            } else {
                args.push(arg);
            }
        }

        // Get ignore case from env var
        let env_ignore_case: &str = &env::var(OPT_IGNORE_CASE).unwrap_or("0".to_string());

        // Ensure args_only has two args
        match args.len().cmp(&2) {
            Ordering::Greater => return Err(format!("Unknown argument: {}", &args[2])),
            Ordering::Less => return Err("Usage: minigrep [OPTS] PATTERN FILE_PATH".to_string()),
            _ => (),
        }

        // Default config
        let mut config = Config {
            query: args[0].to_string(),
            file_path: args[1].to_string(),
            ignore_case: ["true", "1", "yes"].contains(&env_ignore_case),
        };

        // Set options and return
        for opt in opts {
            config.map_option(opt)?
        }
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Convert a string command into a iterator like env::args()
    fn get_args(command: &str) -> impl Iterator<Item = String> + '_ {
        command.split_whitespace().map(|s| s.to_string())
    }

    #[test]
    fn unknown_args_should_error() {
        let args = get_args("/minigrep my_pattern path/to/file.ext extra arguments");
        let config = Config::build(args);
        assert_eq!(config, Err("Unknown argument: extra".to_string()));
    }

    #[test]
    fn unknown_opts_should_error() {
        let args = get_args("/minigrep my_pattern path/to/file.ext -g --other");
        let config = Config::build(args);
        assert_eq!(config, Err("Unknown option: -g".to_string()));
    }

    #[test]
    fn args_parse_only_args() {
        let args = get_args("/minigrep my_pattern path/to/file.ext");
        let config = Config::build(args).unwrap();

        assert_eq!(config.query, "my_pattern", "Query not parsed correctly");
        assert_eq!(
            config.file_path, "path/to/file.ext",
            "File path not parsed correctly"
        );
        assert_eq!(
            config.ignore_case, false,
            "Ignore case shouldn't be enabled"
        );
    }

    #[test]
    fn args_parse_option_last() {
        let args = get_args("/minigrep my_pattern path/to/file.ext -i");
        let config = Config::build(args).unwrap();

        assert_eq!(config.query, "my_pattern", "Query not parsed correctly");
        assert_eq!(
            config.file_path, "path/to/file.ext",
            "File path not parsed correctly"
        );
        assert_eq!(
            config.ignore_case, true,
            "Ignore case wasn't enabled by option"
        );
    }

    #[test]
    fn args_parse_option_first() {
        let args = get_args("/minigrep -i my_pattern path/to/file.ext");
        let config = Config::build(args).unwrap();

        assert_eq!(config.query, "my_pattern", "Query not parsed correctly");
        assert_eq!(
            config.file_path, "path/to/file.ext",
            "File path not parsed correctly"
        );
        assert_eq!(
            config.ignore_case, true,
            "Ignore case wasn't enabled by option"
        );
    }

    #[test]
    fn args_parse_option_middle() {
        let args = get_args("/minigrep my_pattern -i path/to/file.ext");
        let config = Config::build(args).unwrap();

        assert_eq!(config.query, "my_pattern", "Query not parsed correctly");
        assert_eq!(
            config.file_path, "path/to/file.ext",
            "File path not parsed correctly"
        );
        assert_eq!(
            config.ignore_case, true,
            "Ignore case wasn't enabled by option"
        );
    }
}
