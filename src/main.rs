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

pub(crate) use monsters::Monsters;
pub(crate) use emojis::Emojis;
pub(crate) use error::{ Error, Result };

use std::process;
use chrono::Local;

fn main() {
    env_logger::init();

    info!("Start drakeema");

    let matches = clap::App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .arg(
            clap::Arg::with_name("listen")
                .short("l")
                .long("listen")
                .case_insensitive(false)
                .help("Listen to some timelines and react to toots that contain some keywords")

        )
        .arg(
            clap::Arg::with_name("announce")
                .short("a")
                .long("announce")
                .case_insensitive(false)
                .help("Announce information of some contents in Astoltia")
        )
        .group(
            clap::ArgGroup::with_name("mode")
            .args(&["listen", "announce"])
            .required(true)
        )
        .get_matches();

    if matches.is_present("announce") {
        info!("Start announcement");

        match features::announcement::announce(&features::AnnouncementCriteria::new(Local::now())) {
            Ok(_) => info!("Announcement completed"),
            Err(e) => error!("{}", e),
        };
    } else if matches.is_present("listen") {
        info!("Start to listen timelines");

        match features::bot::attach() {
            Ok(_) => info!("Timeline listening completed"),
            Err(e) => error!("{}", e),
        }
    }

    info!("Exit drakeema");
    process::exit(0);
}
