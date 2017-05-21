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
    pub columns_count: u64,
    pub rows_count: u64,
    pub fields_count: u64
}

impl SourceStats {
    pub fn process_end_of_stream(&self) {
        debug!("Stats: {:?}", &self);
    }

    pub fn process_header(&mut self, data: &Option<Vec<String>>) {
        if let &Some(ref data) = data {
            self.columns_count = data.len() as u64;
        }

        self.process_row(data);
        debug!("{:?}", data);
    }

    pub fn process_row(&mut self, data: &Option<Vec<String>>) {
        if let &Some(ref _data) = data {
            self.rows_count += 1;
            self.fields_count += self.columns_count;
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
        .arg(Arg::with_name("FILE")
            .help("Files to process")
            .index(1)
            .required(true)
            .multiple(true))
        .get_matches();

    let _opts = Options::from(&matches);

    // TODO: Use debug/release(null) implementation of logger
    match matches.occurrences_of("verbose") {
        0 => {}
        1 => env::set_var("RUST_LOG", "warn"),
        2 => env::set_var("RUST_LOG", "info"),
        _ => env::set_var("RUST_LOG", "debug"),
    }

    env_logger::init().unwrap();

    let _ = source::file::Csv::new();

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
    //  Transpose,
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
    // - Collect & Interpret result

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
    let workers = files.iter().map(move |file| {
        let tx = tx.clone();

        // TODO: Use only one IO thread
        thread::spawn(move || {
            if let Ok(mut rdr) = csv::Reader::from_file(file) {
                let _ = tx.send((MessageType::Header, Some(rdr.headers().unwrap())));

                for record in rdr.records() {
                    if let Ok(record) = record {
                        let _ = tx.send((MessageType::Row, Some(record)));
                    }
                }

                let _ = tx.send((MessageType::EndOfStream, None));
            }
        })
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
        }
    }

    let res = workers.map(|worker| {
        worker.join();
    });

    println!("Done - {:?}", res);
}