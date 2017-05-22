use clap::ArgMatches;

pub mod csv;

use self::csv::OptionsCsv;

pub const DEFAULT_BULK_SIZE: usize = 64;
pub const DEFAULT_CHANNEL_SIZE: usize = 64;
pub const DEFAULT_DELIMITER: u8 = b',';

#[derive(Debug, Clone)]
pub struct Options {
    pub csv: OptionsCsv
}

impl<'a> From<&'a ArgMatches<'a>> for Options {
    fn from(matches: &ArgMatches) -> Options {
        debug!("Parsing options");

        Options {
            csv: OptionsCsv {
                delimiter: match matches.value_of("delimiter") {
                    Some(val) => val.to_string().bytes().nth(0).unwrap_or(DEFAULT_DELIMITER),
                    _ => DEFAULT_DELIMITER
                },
                has_header: matches.is_present("has-header"),
                flexible: matches.is_present("flexible"),
            },
        }
    }
}

