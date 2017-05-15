use clap::ArgMatches;

#[derive(Debug, Default, Clone)]
pub struct Options {
}

impl<'a> From<&'a ArgMatches<'a>> for Options {
    fn from(matches: &ArgMatches) -> Options {
        debug!("Parsing options");

        Options::default()
    }
}

