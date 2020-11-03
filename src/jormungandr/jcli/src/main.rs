extern crate bech32;
extern crate chain_addr;
extern crate chain_core;
extern crate chain_crypto;
extern crate chain_impl_mockchain;
extern crate chain_time;
extern crate gtmpl;
extern crate hex;
extern crate jormungandr_lib;
#[cfg(test)]
#[macro_use]
extern crate maplit;
extern crate mime;
extern crate openapiv3;
extern crate rand;
extern crate rand_chacha;
extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate serde_yaml;
extern crate structopt;
extern crate valico;

mod jcli_app;

use std::error::Error;
use structopt::StructOpt;

fn main() {
    jcli_app::JCli::from_args()
        .exec()
        .unwrap_or_else(report_error)
}

fn report_error(error: Box<dyn Error>) {
    eprintln!("{}", error);
    let mut source = error.source();
    while let Some(sub_error) = source {
        eprintln!("  |-> {}", sub_error);
        source = sub_error.source();
    }
    std::process::exit(1)
}