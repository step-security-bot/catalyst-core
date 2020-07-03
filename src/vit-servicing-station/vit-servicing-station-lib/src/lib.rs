#[macro_use]
extern crate diesel;
#[macro_use]
extern crate structopt;

#[cfg(test)]
#[macro_use]
extern crate diesel_migrations;

pub mod db;
pub mod server;
pub mod utils;
pub mod v0;
