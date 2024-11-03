mod clock;
mod ntp;

use chrono::DateTime;
use clap::{Args, Parser, Subcommand, ValueEnum};

use clock::Clock;

/// Get/set the system time using NTP.
#[derive(Parser)]
#[command(name = "ntp-clock")]
#[command(version, about, long_about=None)]
struct ArgParser {
    #[command(subcommand)]
    command: Option<CliAction>,
}

#[derive(Subcommand)]
enum CliAction {
    /// Get the current system time
    Get(GetDateTimeArgs),

    /// Set the current system time
    Set(SetDateTimeArgs),
}

#[derive(Args)]
struct GetDateTimeArgs {
    /// Standard to use when formatting the datetime.
    #[arg(short, long, value_enum, default_value_t = FormatStandard::Rfc3339)]
    standard: FormatStandard,

    /// Use an NTP request instead of the system time
    #[arg(short, long, default_value_t = false)]
    use_ntp: bool,
}

#[derive(Args)]
struct SetDateTimeArgs {
    /// Standard to use when formatting the datetime.
    #[arg(short, long, value_enum, default_value_t = FormatStandard::Rfc3339)]
    standard: FormatStandard,

    /// Datetime to set in the system clock
    datetime: String,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum FormatStandard {
    /// Number of seconds since the UNIX epoch
    Timestamp,
    /// RFC 2822 for email message headers
    Rfc2822,
    /// RFC 3339 commonly associated with ISO 8601
    Rfc3339,
}

fn main() {
    let args = ArgParser::parse();

    if let Some(command) = args.command {
        match command {
            CliAction::Get(get_args) => {
                let datetime = if get_args.use_ntp {
                    Clock::get_from_ntp()
                } else {
                    Clock::get_local()
                };

                let datetime = match get_args.standard {
                    FormatStandard::Timestamp => datetime.timestamp().to_string(),
                    FormatStandard::Rfc2822 => datetime.to_rfc2822(),
                    FormatStandard::Rfc3339 => datetime.to_rfc3339(),
                };
                println!("{}", datetime);
            }
            CliAction::Set(set_args) => {
                let datetime = set_args.datetime;
                let datetime = match set_args.standard {
                    FormatStandard::Timestamp => DateTime::parse_from_str(&datetime, "%s"),
                    FormatStandard::Rfc2822 => DateTime::parse_from_rfc2822(&datetime),
                    FormatStandard::Rfc3339 => DateTime::parse_from_rfc3339(&datetime),
                };

                if let Ok(datetime) = datetime {
                    match Clock::set(datetime) {
                        Ok(_) => println!("System time changed successfully to {}", datetime),
                        Err(err) => eprintln!("ERROR: {}: {}", err, err.kind()),
                    };
                } else {
                    eprintln!("ERROR: Failed parsing the datetime");
                }
            }
        }
    } else {
        println!("{}", Clock::get_local());
    }
}
