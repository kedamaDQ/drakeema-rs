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
    Error,
	Monsters,
	Result,
	contents,
	listeners::{
        LocalTimelineListener,
        UserTimelineListener,
    },
};
use super::{
	Reaction,
	ReactionCriteria,
};

const KEEMASAN_REGEX: &str = "キーマさん";
const OSHIETE_REGEX: &str = "(?:(?:おし|教)えて|(?:てぃーち|ティーチ|ﾃｨーﾁ)\\s*(?:みー|ミー|ﾐー))";
const MAX_RETRY: usize = 5;

pub fn attach() -> Result<()> {
	let conn = Connection::from_file(crate::ENV_FILE)?;
    let me = Arc::new(accounts::verify_credentials::get(&conn).send()?);
    let (tx, rx) = mpsc::channel();

    let tx_for_local = mpsc::Sender::clone(&tx);
    let me_for_local = Arc::clone(&me);
    thread::spawn(move || {
        let listener = LocalTimelineListener::new(&me_for_local, tx_for_local);
        if let Err(e) = listen(StreamType::PublicLocal, &listener) {
            error!("The thread for listening to the public local timeline is dead: {}", e);
            process::exit(1);
        }
    });

    let conn_for_user = Connection::from_file(crate::ENV_FILE)?;
    let tx_for_user = mpsc::Sender::clone(&tx);
    let me_for_user = Arc::clone(&me);
    thread::spawn(move || {
        let listener = UserTimelineListener::new(&conn_for_user, &me_for_user, tx_for_user);
        if let Err(e) = listen(StreamType::User, &listener) {
            error!("The thread for listening to the user timeline is dead: {}", e);
            process::exit(1);
        }
    });

    use std::str::FromStr;

    let monsters = Monsters::load()?;
    let jashin = contents::Jashin::load(&monsters)?;
    let seishugosha = contents::Seishugosha::load(&monsters)?;
    let boueigun = contents::Boueigun::load(&monsters)?;

    let reactions: Vec<&dyn Reaction> = vec![
        &jashin,
        &seishugosha,
        &boueigun,
    ];

    let mut emojis = Emojis::load(&conn).unwrap();
    let keemasan = regex::Regex::from_str(KEEMASAN_REGEX).unwrap();
    let oshiete = regex::Regex::from_str(OSHIETE_REGEX).unwrap();

    for message in rx {
        let status = match message {
            Ok(status) => status,
            Err(e) => return Err(e),
        };
        let content = match status.content() {
            Some(content) => content,
            None => continue,
        };

        if keemasan.is_match(content) && oshiete.is_match(content) {
            let mut response = reactions.iter()
                .map(|i| i.reaction(&ReactionCriteria::new(chrono::Local::now(), content)))
                .filter(|i| i.is_some())
                .map(|i| i.unwrap())
                .collect::<Vec<String>>()
                .join("\n");

            if response.is_empty() {
                response = String::from("？");
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

fn listen(
    stream_type: StreamType,
    listener: &impl EventListener,
) -> Result<()> {
    let conn = Connection::from_file(crate::ENV_FILE)?;
    let mut stream = streaming::get(&conn, stream_type.clone()).send()?;
    let mut retry = 0;
    while retry < MAX_RETRY {
        match stream.attach(listener) {
            Ok(_) => {
                if retry != 0 {
                    streaming::get(&conn, stream_type.clone()).send()?;
                }
            },
            Err(e) => {
                retry += 1;
                error!("{} timeline listener returns an error: {}, retry: {}", stream_type, e, retry);
            }
        };
    }
    Err(Error::LostStreamingConnectionError(stream_type, MAX_RETRY))
}
