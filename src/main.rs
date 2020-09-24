#[macro_use]
extern crate log;

#[macro_use]
extern crate lazy_static;

pub(crate) mod contents;
pub(crate) mod emojis;
pub(crate) mod error;
pub(crate) mod features;
pub(crate) mod listeners;
pub(crate) mod monsters;
pub(crate) mod rate_limit;
pub(crate) mod resistances;
pub(crate) mod tmp_file;
pub(crate) mod utils;
pub(crate) mod message_processor;

pub(crate) use error::{ Error, Result };
pub(crate) use monsters::Monsters;
pub(crate) use message_processor::Message;

use std::process;
use std::sync::mpsc;
use mastors::prelude::*;
use features::announcement::{
	ContentsWorker,
	FeedsWorker,
};
use features::response::ResponseWorker;
use message_processor::MessageProcessor;

lazy_static! {
	static ref MONSTERS: Monsters = {
		monsters::Monsters::load().unwrap()
	};
}

fn monsters() -> &'static Monsters {
	&MONSTERS
}

fn main() {
	if parse_args().is_present("notime") {
		env_logger::builder().format_timestamp(None).init();
	} else {
		env_logger::init();
	}
	info!("Start drakeema: {}", env!("CARGO_PKG_VERSION"));

	let contents_worker = match ContentsWorker::load() {
		Ok(cw) => cw,
		Err(e) => {
			error!("Fatal error occurred while initialize ContentsWorker: {}", e);
			process::exit(1);
		},
	};

	let feeds_worker = match FeedsWorker::load() {
		Ok(fw) => fw,
		Err(e) => {
			error!("Fatal error occurred while initialize FeedsWorker: {}", e);
			process::exit(1);
		},
	};

	let response_worker = match ResponseWorker::load() {
		Ok(rw) => rw,
		Err(e) => {
			error!("Fatal error occurred while initialize ResponseWorker: {}", e);
			process::exit(1);
		},
	};

	let conn = match Connection::new() {
		Ok(conn) => conn,
		Err(e) => {
			error!("Fatal error occurred while create Connection for mastors: {}", e);
			process::exit(1);
		},
	};

	let mut processor = match MessageProcessor::new(&conn) {
		Ok(mp) => mp,
		Err(e) => {
			error!("Fatal error occurred while initialize MessageProcessor: {}", e);
			process::exit(1);
		},
	};

	let (tx, rx) = mpsc::channel();

	contents_worker.start(mpsc::Sender::clone(&tx));
	feeds_worker.start(mpsc::Sender::clone(&tx));
	response_worker.start(mpsc::Sender::clone(&tx));

	for message in rx {
		if let Err(e) = processor.process(message) {
			error!("A fatal error has occurred while processing message: {}", e);
			process::exit(9);
		}
	}

	info!("Exit drakeema");
	process::exit(0);
}

fn parse_args<'a>() -> clap::ArgMatches<'a> {
    clap::App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .arg(
            clap::Arg::with_name("time")
                .short("t")
                .long("time")
                .case_insensitive(false)
                .help("Write time stamp to the log")

        )
        .arg(
            clap::Arg::with_name("notime")
                .short("T")
                .long("no-time")
                .case_insensitive(false)
                .help("Do not write time stamp to the log")
        )
        .group(
            clap::ArgGroup::with_name("logmode")
            .args(&["time", "notime"])
            .required(false)
        )
        .get_matches()
}
