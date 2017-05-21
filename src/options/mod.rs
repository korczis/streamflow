use clap::ArgMatches;

pub mod csv;

use self::csv::OptionsCsv;

#[derive(Debug, Default, Clone)]
pub struct Options {
    pub csv: OptionsCsv
}

impl<'a> From<&'a ArgMatches<'a>> for Options {
    fn from(_matches: &ArgMatches) -> Options {
        debug!("Parsing options");

        Options::default()
    }
}

