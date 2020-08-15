use std::process;
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;
use mastors::prelude::*;
use mastors::{
	Connection,
	api::v1::accounts,
	api::v1::statuses,
	api::v1::streaming,
};
use crate::{
	Emojis,
	Monsters,
	Result,
	contents,
	listeners,
};
use super::{
	Reaction,
	ReactionCriteria,
};

const KEEMASAN_REGEX: &str = "キーマさん";
const OSHIETE_REGEX: &str = "(?:(?:おし|教)えて|(?:てぃーち|ティーチ|ﾃｨーﾁ)\\s*(?:みー|ミー|ﾐー))";

pub fn attach() -> Result<()> {
	let conn = Connection::from_file(crate::ENV_FILE)?;
    let me = Arc::new(accounts::verify_credentials::get(&conn).send()?);
    let monsters = Monsters::load().unwrap();
    let jashin = contents::Jashin::load(&monsters).unwrap();
    let seishugosha = contents::Seishugosha::load(&monsters).unwrap();
    let boueigun = contents::Boueigun::load(&monsters).unwrap();

    let reactions: Vec<&dyn Reaction> = vec![
        &jashin,
        &seishugosha,
        &boueigun,
    ];

    let (tx, rx) = mpsc::channel();

    let tx_for_local = mpsc::Sender::clone(&tx);
    let me_for_local = Arc::clone(&me);
    thread::spawn(move || {
        let conn = get_conn();
        let mut stream = match streaming::get(&conn, StreamType::PublicLocal).send() {
			Ok(stream) => stream,
			Err(e) => {
                error!("Failed to attach to the user timeline: {}", e);
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
    thread::spawn(move || {
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

    let mut emojis = Emojis::load(&conn).unwrap();
    let keemasan = regex::Regex::from_str(KEEMASAN_REGEX).unwrap();
    let oshiete = regex::Regex::from_str(OSHIETE_REGEX).unwrap();

    for status in rx {
        let content = match status.content() {
            Some(content) => content,
            None => continue,
        };

        if keemasan.is_match(content) && oshiete.is_match(content) {
            let response = reactions.iter()
                .map(|i| i.reaction(&ReactionCriteria::new(chrono::Local::now(), content)))
                .filter(|i| i.is_some())
                .map(|i| i.unwrap())
                .collect::<Vec<String>>()
                .join("\n");

            if response.is_empty() {
                continue;
            }

            let response = emojis.emojify(String::from("@") + status.account().acct() + "\n\n" + response.as_str());

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

    info!("Exit drakeema");
    process::exit(0);

}

fn get_conn() -> Connection {
    match Connection::from_file(crate::ENV_FILE) {
        Ok(conn) => conn,
        Err(e) => {
            error!("Error: Faild to load env: {}", e);
            process::exit(1);
        },
    }
}
