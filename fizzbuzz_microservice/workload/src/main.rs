extern crate clap;
extern crate futures;
extern crate hyper;
extern crate rand;
extern crate serde;
extern crate tokio_core;
extern crate tokio_timer;

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

use std::str::FromStr;
use std::str::from_utf8;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use clap::{Arg, App};
use futures::future;
use futures::{Future, Stream};
use hyper::client::Client;
use hyper::error::Error;
use hyper::{Method, Response, Request, Uri};
use rand::{StdRng, Rng};
use tokio_core::reactor::Core;
use tokio_timer::wheel;
use tokio_timer::TimerError;


fn main() {
    let opts = App::new("Workload").arg(Arg::with_name("rate")
                                            .short("r")
                                            .long("rate")
                                            .help("The rate to make requests with integers.")
                                            .takes_value(true))
                                      .arg(Arg::with_name("update")
                                            .short("u")
                                            .long("update")
                                            .help("The number of seconds between printing to stdout.")
                                            .takes_value(true))
                                      .arg(Arg::with_name("target")
                                            .short("t")
                                            .long("target")
                                            .help("The url of the service to connect to.")
                                            .takes_value(true))
                                      .get_matches();

    let rate = u64::from_str_radix(opts.value_of("rate").unwrap_or("10"), 10).unwrap_or(10);
    let time_between_updates = u64::from_str_radix(opts.value_of("update").unwrap_or("15"), 10).unwrap_or(15);
    let target = opts.value_of("target").unwrap_or("http://localhost:8080");

    let time_to_wait = 1_000_000_000 / rate as u32;

    println!("Rate {}, Time between updates {}", rate, time_between_updates);
    
    let result = run_workload(time_to_wait, target);
    match result {
        Ok(result) => println!("Worked"),
        Err(err) => println!("{:?}", err),
    }
    
}

fn run_workload(time_to_wait: u32, target: &str) -> Result<(), Error> {
    let mut int_iterator = IntIterator::new();  

    let mut core = Core::new().unwrap();
    let uri: Uri = Uri::from_str(target).unwrap();
    let handle = &core.handle();
    
    let timer = wheel().build();
    println!("Running Timeout");
    let work = timer.interval(Duration::new(0, time_to_wait)).map_err(|e| {Error::Incomplete})
        .map(|()| int_iterator.next())
        .and_then(|num:Option<u64>| {
            println!("About to make request.");
            match num {
                Some(num) => {
                    let mut req = Request::new(Method::Post, uri.clone());
                    req.set_body(json!({"input": num}).to_string());
                    println!("Some {}", num);
                    Client::new(handle).request(req)
                },
                None => panic!("There was an error generating an integer."),
            }
        }).and_then(|response:Response| {
            println!("What?");
            response.body().concat2()
        }).map(|body| {
            let output: ServerOutput = serde_json::from_str(from_utf8(&body.to_vec())?)?;
            println!("Output {}", output.output);
            Ok(())
        }).for_each(|_:Result<(),WorkloadError>|{println!("hello"); future::ok(())});
    core.run(work)
}

#[derive(Serialize)]
struct ServerInput {
    input: u64,
}

#[derive(Deserialize)]
struct ServerOutput {
    output: u64,
}

#[derive(Debug)]
enum WorkloadError {
    IOError,
    JSONError,
    HTTPError,
    OSError,
}

impl From<TimerError> for WorkloadError {
    fn from(err: TimerError) -> Self {
        WorkloadError::OSError
    }
}

impl From<serde_json::Error> for WorkloadError {
    fn from(err: serde_json::Error) -> Self {
        WorkloadError::JSONError
    }
}

impl From<std::io::Error> for WorkloadError {

    fn from(err: std::io::Error) -> Self {
        WorkloadError::IOError
    }
}

impl From<Error> for WorkloadError {
    fn from(err: Error) -> Self {
        WorkloadError::HTTPError
    }
}

impl From<std::str::Utf8Error> for WorkloadError {
    fn from(err: std::str::Utf8Error) -> Self {
        WorkloadError::JSONError
    }
}

#[derive(Debug)]
struct IntIterator {

    rng: StdRng,
}

impl IntIterator {

    fn new() -> Self {
        IntIterator {
            rng: StdRng::new().unwrap(),
        }
    }
}

impl Iterator for IntIterator {
    type Item = u64;


    fn next(&mut self) -> Option<u64> {
        Some(self.rng.next_u64())
    }
}

#[derive(Debug)]
struct Counter {
    three: AtomicUsize, // Count of the multiples of three.
    five: AtomicUsize, // Count of the multiples of five.
    three_and_five: AtomicUsize, // Count of the multiple of three and five.
}

impl Counter {

    fn new() -> Self {
        Counter {
            three: AtomicUsize::new(0),
            five: AtomicUsize::new(0),
            three_and_five: AtomicUsize::new(0),
        }
    }

    fn fizz(&self) {
        self.three.fetch_add(1, Ordering::Relaxed);
    }

    fn buzz(&self) {
        self.five.fetch_add(1, Ordering::Relaxed);
    }

    fn fizzbuzz(&self) {
        self.three_and_five.fetch_add(1, Ordering::Relaxed);
    }

    fn zero(&self) {
        self.three.store(0, Ordering::Relaxed);
        self.five.store(0, Ordering::Relaxed);
        self.three_and_five.store(0, Ordering::Relaxed);
    }
}
