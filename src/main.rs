#[macro_use]
extern crate log;
extern crate env_logger;

extern crate clap;
extern crate csv;
extern crate streamflow;
extern crate time;

use clap::{App, Arg};
// use streamflow::*;
use std::env;
use std::sync::mpsc;
use std::thread;
// use std::fs;
// use std::os::unix::fs::MetadataExt;
// use time::PreciseTime;

use streamflow::data::source;
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
            .required(true)
            .multiple(true))
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

    let _ = source::file::Csv::new();

    let (tx, rx) = mpsc::channel();

    enum MessageType {
        Header,
        Row,
        EndOfStream
    };

    let files: Vec<_> = match matches.values_of("FILE") {
        Some(dirs) => dirs.map(|d| d.to_string()).collect(),
        _ => vec![String::from(".")],
    };

    // TODO: Use generic URI (Connection String)
    // - file://.tmp/incidents.csv?has-header=true?delimiter="|"
    // - http://whatever.net/data.csv

    // TODO: Wrap in method - crate_worker_threads()
    let thread_handle = thread::spawn(move || {
        for file in files {
            debug!("Processing file {:?}", file);
            if let Ok(rdr) = csv::Reader::from_file(file) {
                let mut rdr = rdr.delimiter(opts.csv.delimiter)
                    .has_headers(opts.csv.has_header)
                    .flexible(opts.csv.flexible);

                if opts.csv.has_header {
                    let _ = tx.send((MessageType::Header, Some(rdr.headers().unwrap())));
                }

                for record in rdr.records() {
                    if let Ok(record) = record {
                        let _ = tx.send((MessageType::Row, Some(record)));
                    }
                }

                let _ = tx.send((MessageType::EndOfStream, None));
            }
        }
    });

    let mut stats = SourceStats::default();

    // TODO: Wrap in method - crate_processing_thread()
    while let Ok(data) = rx.recv() {
        let (mt, data) = data;

        match mt {
            MessageType::Header => {
                stats.process_header(&data);
            }
            MessageType::Row => {
                stats.process_row(&data);
            }
            MessageType::EndOfStream => {
                stats.process_end_of_stream();
                break;
            }
        };
    }

    let _ = thread_handle.join();
    debug!("Done");
}