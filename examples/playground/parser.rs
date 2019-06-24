use std::ffi::{OsStr, OsString};

use osmgraphing::osm;

fn parse_pbf<S: AsRef<OsStr> + ?Sized>(path: &S) {
    let parser = osm::pbf::Parser;
    let graph = parser
        .parse(&path)
        .expect("PBF-Parser should parse given pbf-file.");
    println!("{}", graph);
}

fn parse_fmi<S: AsRef<OsStr> + ?Sized>(path: &S) {
    let parser = osm::fmi::Parser;
    let graph = parser
        .parse(&path)
        .expect("FMI-Parser should parse given fmi-file.");
    println!("{}", graph);
}

fn main() {
    let filename = match std::env::args_os().nth(1) {
        Some(filename) => filename,
        None => OsString::from("resources/osm/small.fmi"),
    };

    // check if filetype is supported
    match osm::Support::from_path(&filename) {
        Ok(osm::Support::PBF) => parse_pbf(&filename),
        Ok(osm::Support::FMI) => parse_fmi(&filename),
        Err(e) => panic!("{:}", e),
    };
}
