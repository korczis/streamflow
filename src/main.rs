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
// use time::PreciseTime;
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

    //    let (tx, rx) = mpsc::channel();

    #[derive(Debug)]
    enum MessageType {
        Header,
        Row,
        EndOfStream
    };

    let files: Vec<_> = match matches.values_of("FILE") {
        Some(dirs) => dirs.map(|d| d.to_string()).collect(),
        _ => vec![String::from(".")],
    };

    // README: This is based on http://hermanradtke.com/2017/03/03/future-mpsc-queue-with-tokio.html

    // tokio Core is an event loop executor. An executor is what runs a future to
    // completion.
    let mut core = Core::new().expect("Failed to create core");

    // `core.remote()` is a thread safe version of `core.handle()`. Both `core.remote()`
    // and `core.handle()` are used to spawn a future. When a future is _spawned_,
    // it basically means that it is being executed.
    let remote = core.remote();

    // Now we create a multi-producer, single-consumer channel. This channel is very
    // similar to the mpsc channel in the std library. One big difference with this
    // channel is that `tx` and `rx` return futures. In order to have `tx` or `rx`
    // actually do any work, they have to be _executed_ by Core.
    //
    // The parameter passed to `mpsc::channel()` determines how large the queue is
    // _per tx_. Since we are cloning `tx` per iteration of the loop, we are guranteed
    // 1 spot for each loop iteration. Cloning tx is how we get multiple producers.
    //
    // For more detail on mpsc, see https://tokio.rs/docs/going-deeper/synchronization/
    //
    // Quick note:
    //    - `tx` is of type `Sink`. A sink is something that you can place a value into
    //    and then _flush_ the value into the queue.
    //    - `rx` is of type `Stream`. A stream is an iterator of _future_ values.
    // More details on `tx` and `rx` below. For even more detail, see
    // https://tokio.rs/docs/getting-started/streams-and-sinks/
    let (tx, rx) = mpsc::channel(10);

    let mut stats = SourceStats::default();

    // Create a thread that performs some work.
    let thread_handle = thread::spawn(move || {
        for file in files {
            let tx = tx.clone();

            if let Ok(rdr) = csv::Reader::from_file(file) {
                let mut rdr = rdr.delimiter(opts.csv.delimiter)
                    .has_headers(opts.csv.has_header)
                    .flexible(opts.csv.flexible);

                for record in rdr.records() {
                    let tx = tx.clone();

                    // `remote.spawn` accepts a closure with a single parameter of type `&Handle`.
                    // In this example, the `&Handle` is not needed. The future returned from the
                    // closure will be executed.
                    //
                    // Note: We must use `remote.spawn()` instead of `handle.spawn()` because the
                    // Core was created on a different thread.
                    remote.spawn(|_| {
                        tx.send((MessageType::Row, Some(record.unwrap())))
                            .then(|tx| {
                                match tx {
                                    Ok(_tx) => {
                                        // info!("Sink flushed");
                                        Ok(())
                                    }
                                    Err(e) => {
                                        error!("Sink failed! {:?}", e);
                                        Err(())
                                    }
                                }
                            })
                    })
                }
            }
        }
    });

    // As mentioned above, rx is a stream. That means we are expecting multiple _future_
    // values. Here we use `for_each` to yield each value as it comes through the channel.
    let f2 = rx.for_each(|data| {
        // The stream will stop on `Err`, so we need to return `Ok`.
        let (mt, data) = data;

        match mt {
            MessageType::Header => {
                stats.process_header(&data);
            }
            MessageType::Row => {
                stats.process_row(&data);
            }
            MessageType::EndOfStream => {
                debug!("{:?}", &stats);
                stats.process_end_of_stream();
            }
        };

        Ok(())
    });

    // The executor is started by the call to `core.run()` and will finish once the `f2`
    // future is finished. Keep in mind that since `rx` is a stream, it will not finish
    // until there is an error. Using a stream with `core.run()` is a common pattern and
    // is how servers are normally implemented.
    core.run(f2).expect("Core failed to run");

    let _ = thread_handle.join();
    debug!("Done");
}