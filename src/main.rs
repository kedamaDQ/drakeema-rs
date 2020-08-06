#[macro_use]
extern crate log;

mod contents;
mod emojis;
mod error;
mod listeners;
mod monsters;
mod resistances;
mod utils;

pub use error::{ Error, Result };

use std::process;
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;
use mastors::prelude::*;
use mastors::api::{
    v1::accounts,
    v1::statuses,
    v1::streaming,
};
use contents::{
    Announcement,
    AnnouncementCriteria,
    Information,
    InformationCriteria
};

const KEEMASAN_REGEX: &str = "キーマさん";
const OSHIETE_REGEX: &str = "(?:(?:おし|教)えて|(?:てぃーち|ティーチ|ﾃｨーﾁ)\\s*(?:みー|ミー|ﾐー))";
//const ENV: &str = ".env.test";
const ENV: &str = ".env.test.st";
//const ENV: &str = ".env";

fn main() {
    env_logger::init();

    let conn = get_conn();
    let me = match accounts::verify_credentials::get(&conn).send() {
        Ok(me) => Arc::new(me),
        Err(e) => {
            error!("Failed to verify credentials: {:#?}", e);
            process::exit(2);
        },
    };
    let monsters = monsters::load().unwrap();
    let jashin = contents::Jashin::load(&monsters).unwrap();
    let seishugosha = contents::Seishugosha::load(&monsters).unwrap();

    let informations: Vec<&dyn Information> = vec![
        &jashin,
        &seishugosha,
    ];

    let announcements: Vec<&dyn Announcement> = vec![
        &jashin,
        &seishugosha,
    ];

    let (tx, rx) = mpsc::channel();


    let tx_for_local = mpsc::Sender::clone(&tx);
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
            listeners::LocalTimelineListener::new(&me_for_local, &tx_for_local)
        ) {
            error!("Local timeline listener returns an error: {}", e);
        }
    });

    let tx_for_user = mpsc::Sender::clone(&tx);
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
            listeners::UserTimelineListener::new(&conn, &me_for_user, &tx_for_user)
        ) {
            error!("User timeline listener returns an error: {}", e);
        }
    });

    use std::str::FromStr;

    let keemasan = regex::Regex::from_str(KEEMASAN_REGEX).unwrap();
    let oshiete = regex::Regex::from_str(OSHIETE_REGEX).unwrap();

    for status in rx {
        let content = match status.content() {
            Some(content) => content,
            None => break,
        };

        if keemasan.is_match(content) && oshiete.is_match(content) {
            let response = informations.iter()
                .map(|i| i.information(InformationCriteria::new(chrono::Local::now(), content)))
                .filter(|i| i.is_some())
                .map(|i| i.unwrap())
                .collect::<Vec<String>>()
                .join("\n");

            if response.is_empty() {
                break;
            }

            let response = String::from("@") + status.account().acct() + "\n\n" + response.as_str();

            let mut post = statuses::post(&conn, response);
            if status.account().is_remote() {
                post = post.in_reply_to_id(status.id());
            }

            match post.send() {
                Ok(posted) => info!(
                    "Posted: {}, {:#?}",
                    posted.id(),
                    posted.status().unwrap().content()
                ),
                Err(e) => error!("Can't send status: {}", e),
            };

        } /*else {
            kantan-na-koto
        }
        */
    }

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
