#[macro_use]
extern crate log;

pub(crate) mod contents;
pub(crate) mod emojis;
pub(crate) mod error;
pub(crate) mod features;
pub(crate) mod listeners;
pub(crate) mod monsters;
pub(crate) mod resistances;
pub(crate) mod tmp_file;
pub(crate) mod utils;

pub(crate) use emojis::Emojis;
pub(crate) use error::{ Error, Result };
pub(crate) use monsters::Monsters;

use std::process;

fn main() {
    env_logger::init();

    info!("Start drakeema");

    let matches = clap::App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .arg(
            clap::Arg::with_name("no-response")
                .long("no-response")
                .case_insensitive(false)
                .help("Do not resopnse to status on timelines")

        )
        .arg(
            clap::Arg::with_name("no-announce")
                .long("no-announce")
                .case_insensitive(false)
                .help("Do not announce about contents in Astoltia and news from RSS feeds")
        )
        .group(
            clap::ArgGroup::with_name("mode")
            .args(&["no-response", "no-announce"])
            .required(false)
        )
        .get_matches();

    if !matches.is_present("no-announce") {

    } else if !matches.is_present("listen") {
        match features::response::start() {
            Ok(_) => info!("Timeline listening completed"),
            Err(e) => error!("{}", e),
        }
    }

    info!("Exit drakeema");
    process::exit(0);
}
