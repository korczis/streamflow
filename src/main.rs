#[macro_use]
extern crate log;
extern crate futures;
extern crate env_logger;

extern crate clap;
extern crate csv;
extern crate streamflow;
extern crate time;
extern crate tokio_core;

use clap::{App, Arg};
// use streamflow::*;
use std::env;

use futures::{Stream, Sink, Future};
use futures::sync::mpsc;
// use std::sync::mpsc;
use std::thread;
// use std::fs;
// use std::os::unix::fs::MetadataExt;
use time::PreciseTime;
use tokio_core::reactor::Core;

use streamflow::options::Options;

const AUTHOR: &'static str = env!("CARGO_PKG_AUTHORS");
const DESCRIPTION: &'static str = env!("CARGO_PKG_DESCRIPTION");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Default)]
struct SourceStats {
    pub header: Option<Vec<String>>,
    pub rows_count: u64,
    pub fields_count: u64
}

impl SourceStats {
    pub fn process_end_of_stream(&self) {
        debug!("Stats: {:?}", &self);
    }

    pub fn process_header(&mut self, data: &Option<Vec<String>>) {
        if let Some(ref data) = *data {
            self.header = Some(data.clone());
        } else {
            self.header = None;
        }

        self.process_row(data);
        debug!("{:?}", data);
    }

    pub fn process_row(&mut self, data: &Option<Vec<String>>) {
        if let Some(ref data) = *data {
            self.rows_count += 1;
            self.fields_count += data.len() as u64;
        }
    }
}

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
        .arg(Arg::with_name("delimiter")
            .help("Delimiter")
            .short("d")
            .long("delimiter")
            .default_value(","))
        .arg(Arg::with_name("has-header")
            .help("CSV has header row")
            .long("has-header"))
        .arg(Arg::with_name("flexible")
            .help("Records in the CSV data can have different lengths")
            .long("flexible"))
        .arg(Arg::with_name("FILE")
            .help("Files to process")
            .index(1)
            .required(true))
        .get_matches();

    let opts = Options::from(&matches);

    // TODO: Use debug/release(null) implementation of logger
    match matches.occurrences_of("verbose") {
        0 => {}
        1 => env::set_var("RUST_LOG", "warn"),
        2 => env::set_var("RUST_LOG", "info"),
        _ => env::set_var("RUST_LOG", "debug"),
    }

    env_logger::init().unwrap();

    let file = String::from("tmp/test.csv"); // matches.value_of("FILE").unwrap();

    let start = PreciseTime::now();

    // TODO: Hard work here!

    // tokio Core is an event loop executor. An executor is what runs a future to
    // completion.
    let mut core = Core::new().expect("Failed to create core");

    // `core.remote()` is a thread safe version of `core.handle()`. Both `core.remote()`
    // and `core.handle()` are used to spawn a future. When a future is _spawned_,
    // it basically means that it is being executed.
    let remote = core.remote();

    // let (tx, rx) = mpsc::channel(1024);

    // Create a thread that performs some work.
    let thread_handle = thread::spawn(move || {
        let file = file.clone();
        let opts = opts.clone();

        if let Ok(rdr) = csv::Reader::from_file(&file) {
            let mut rdr = rdr.delimiter(opts.csv.delimiter)
                .has_headers(opts.csv.has_header)
                .flexible(opts.csv.flexible);
        }
    });

    let diff = start.to(PreciseTime::now());
    let elapsed_secs = diff.num_nanoseconds().unwrap() as f64 * 1e-9;

    let _ = thread_handle.join();
    debug!("Elapsed time {:.2}", elapsed_secs);
}
