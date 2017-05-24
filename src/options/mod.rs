use clap::ArgMatches;

pub mod channel;
pub mod csv;

use self::channel::Channel;
use self::csv::Csv;

pub const DEFAULT_BULK_SIZE: usize = 64;
pub const DEFAULT_CHANNEL_SIZE: usize = 64;
pub const DEFAULT_DELIMITER: u8 = b',';

#[derive(Debug, Clone)]
pub struct Options {
    pub channel: Channel,
    pub csv: Csv
}

impl<'a> From<&'a ArgMatches<'a>> for Options {
    fn from(matches: &ArgMatches) -> Options {
        debug!("Parsing options");

        Options {
            channel: Channel {
                size: matches.value_of("channel-size")
                    .unwrap_or(&DEFAULT_CHANNEL_SIZE.to_string())
                    .to_string()
                    .parse::<usize>()
                    .unwrap_or(DEFAULT_CHANNEL_SIZE),
            },
            csv: Csv {
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

