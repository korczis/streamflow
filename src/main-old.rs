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
use std::env;

use futures::{Stream, Sink, Future};
use futures::sync::mpsc;
use std::sync::Arc;
use std::thread;
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

    debug!("{:?}", &matches);

    let opts = Options::from(&matches);

    // TODO: Use debug/release(null) implementation of logger
    match matches.occurrences_of("verbose") {
        0 => {}
        1 => env::set_var("RUST_LOG", "warn"),
        2 => env::set_var("RUST_LOG", "info"),
        _ => env::set_var("RUST_LOG", "debug"),
    }

    env_logger::init().unwrap();

    //    let (tx, rx) = mpsc::channel();

    #[derive(Debug)]
    enum MessageType {
        Header,
        Row,
        EndOfStream
    };
    // README: This is based on http://hermanradtke.com/2017/03/03/future-mpsc-queue-with-tokio.html

    let mut core = Core::new().expect("Failed to create core");

    let remote = core.remote();

    let (tx, rx) = mpsc::channel(10);

    let mut stats = SourceStats::default();

    let file = String::from("tmp/test.csv"); // matches.value_of("FILE").unwrap();

    // Create a thread that performs some work.
    let thread_handle = thread::spawn(move || {
        let tx = tx.clone();

        remote.spawn(move |_| {
            let rdr = csv::Reader::from_file(file).unwrap();

            let mut rdr = rdr.delimiter(opts.csv.delimiter)
                .has_headers(opts.csv.has_header)
                .flexible(opts.csv.flexible);

            let mut records = rdr.records();

            let f = ::futures::done::<(), ()>(Ok(()));
            f.then(move |_res| {
                tx.send((MessageType::Row, Some(records.next().unwrap().unwrap())))
                    .then(move |tx| {
                        tx.unwrap().send((MessageType::Row, Some(records.next().unwrap().unwrap())))
                            .then(move |tx| {
                                match tx {
                                    Ok(_tx) => {
                                        info!("Sink flushed");
                                        Ok(())
                                    }
                                    Err(e) => {
                                        error!("Sink failed! {:?}", e);
                                        Err(())
                                    }
                                }
                            })
                    })
            })
        })
    });

    // As mentioned above, rx is a stream. That means we are expecting multiple _future_
    // values. Here we use `for_each` to yield each value as it comes through the channel.
    let f2 = rx.for_each(|data| {
        // The stream will stop on `Err`, so we need to return `Ok`.
        let (mt, data) = data;

        match mt {
            MessageType::Header => {
                debug!("{:?}", &data);
                stats.process_header(&data);
            }
            MessageType::Row => {
                debug!("{:?}", &data);
                stats.process_row(&data);
            }
            MessageType::EndOfStream => {
                debug!("{:?}", &stats);
                stats.process_end_of_stream();
            }
        };

        Ok(())
    });

    core.run(f2).expect("Core failed to run");

    let _ = thread_handle.join();
    debug!("Done");
}