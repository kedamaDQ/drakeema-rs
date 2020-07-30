#[macro_use]
extern crate log;

use std::process;
use std::sync::Arc;
use std::thread;
use mastors::prelude::*;
use mastors::api::{
    v1::accounts,
    v1::streaming,
};

mod contents;
mod error;
mod listeners;
mod monsters;
mod resistances;
mod utils;

pub use error::{ Error, Result };

//const ENV: &str = ".env.test";
const ENV: &str = ".env.test.st";
//const ENV: &str = ".env";

fn main()  {
    env_logger::init();

    let conn = get_conn();
    let me = match accounts::verify_credentials::get(&conn).send() {
        Ok(me) => Arc::new(me),
        Err(e) => {
            error!("Failed to verify credentials: {}", e);
            process::exit(2);
        },
    };

    let monsters = monsters::load().unwrap();
    let jashin = contents::Jashin::load(&monsters).unwrap();
    println!("{:#?}", jashin);
    println!("{}", jashin.information(chrono::Local::now()));

    let me_for_local = Arc::clone(&me);
    let public_local = thread::spawn(move || {
        let conn = get_conn();
        let mut stream = match streaming::get(&conn, StreamType::PublicLocal).send() {
            Ok(stream) => stream,
            Err(e) => {
                error!("Failed to attach to the local timeline: {}", e);
                process::exit(3);
            }
        };
        while let Err(e) = stream.attach(
            listeners::LocalTimelineListener::new(&conn, &me_for_local)
        ) {
            error!("Local timeline listener returns an error: {}", e);
        }
    });

    let me_for_user = Arc::clone(&me);
    let user = thread::spawn(move || {
        let conn = get_conn();
        let mut stream = match streaming::get(&conn, StreamType::User).send() {
            Ok(stream) => stream,
            Err(e) => {
                error!("Failed to attach to the user timeline: {}", e);
                process::exit(3);
            }
        };
        while let Err(e) = stream.attach(
            listeners::UserTimelineListener::new(&conn, &me_for_user)
        ) {
            eprintln!("User timeline listener returns an error: {}", e);
        }
    });

    match public_local.join() {
        Ok(what) => info!("Local timeline thread end with normal status: {:#?}", what),
        Err(e) => error!("Local timeline thread end with abnormal status: {:#?}", e)
    };

    match user.join() {
        Ok(what) => info!("User timeline thread end with normal status: {:#?}", what),
        Err(e) => error!("User timeline thread end with abnormal status: {:#?}", e)
    };

    info!("Exit drakeema");
    process::exit(0);
}

fn get_conn() -> Connection {
    match Connection::from_file(ENV) {
        Ok(conn) => conn,
        Err(e) => {
            error!("Error: Faild to load env: {}", e);
            process::exit(1);
        },
    }
}
