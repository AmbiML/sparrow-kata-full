extern crate zmodem;

extern crate clap;
extern crate env_logger;
extern crate log;

mod stdinout;

use clap::{App, Arg};
use std::fs::File;
use std::path::Path;

fn main() {
    env_logger::init().unwrap();

    let matches = App::new("Pure Rust implementation of rz utility")
        .arg(Arg::with_name("file").required(false).index(1))
        .get_matches();

    let fileopt = matches.value_of("file").unwrap_or("rz-out");
    let filename = Path::new(fileopt).file_name().unwrap().clone();
    let file = File::create(filename).expect(&format!("Cannot create file {:?}:", filename));

    let inout = stdinout::CombinedStdInOut::new();
    zmodem::recv::recv(inout, file).unwrap();
}
