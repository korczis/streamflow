#[macro_use]
extern crate log;
extern crate env_logger;

extern crate clap;
extern crate csv;
extern crate streamflow;
extern crate time;

use clap::{App, Arg};
use streamflow::*;
use std::env;
use std::fs;
use std::os::unix::fs::MetadataExt;
use time::PreciseTime;

use streamflow::options::Options;

const AUTHOR: &'static str = env!("CARGO_PKG_AUTHORS");
const DESCRIPTION: &'static str = env!("CARGO_PKG_DESCRIPTION");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    let matches = App::new(DESCRIPTION)
        .version(VERSION)
        .author(AUTHOR)
        .about("Fast Stream Processing")
        .arg(Arg::with_name("verbose")
            .help("Verbose mode")
            .short("v")
            .long("verbose")
            .multiple(true))
        .get_matches();

    let opts = Options::from(&matches);

    match matches.occurrences_of("verbose") {
        0 => {}
        1 => env::set_var("RUST_LOG", "warn"),
        2 => env::set_var("RUST_LOG", "info"),
        _ => env::set_var("RUST_LOG", "debug"),
    }

    env_logger::init().unwrap();

    let _ = data::source::file::csv::Csv::default();

    // Commands Alternatives
    // - streamflow wc -l ./data/sample.csv

    // Real World:
    // Read CSV
    //  Generate Meta-information (Manifest) [OPTIONAL]
    // Transpose
    // Per column streaming
    // Per stream stat
    // Aggregate

    // Iterators in Functions
    // iterator -> f(context, data) -> iterator

    // let reducer_stats = data::reducer::Stats();
    // let source_csv = data::producer::file::Csv::new({});

    // Reverse notation:
    // reducer_stats
    // .from(source_csv)
    // .from(network_reader)
    // .end();

    // Forward notation:
    // network_reader
    // .to(source_csv)
    // .to(reducer_stats)

    // let flow_manager = FlowManager::default();
    // flow_manager.run([
    //  ReadCsv,
    //  Transponse,
    //  Serialize,
    // ]);

    // TODO: Create transpose block
    // - Create channels
    // - Create create thread
    // - Run loop

    // TODO: Create CSV reading block
    // - Run loop
    //   - Send each row to first block
    // - Wait for chain to finish
    // - Collect & Interpet result
}
