use std::fs::File;
use std::io::BufReader;
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
use serde::Deserialize;
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
    utils::transform_string_to_regex,
};
use super::{
	Reaction,
	ReactionCriteria,
};

const CONFIG: &str = "drakeema-data/drakeema.json";
const MAX_RETRY: usize = 5;

pub fn attach() -> Result<()> {
    let config: Config = serde_json::from_reader(
        BufReader::new(File::open(CONFIG)?)
    )
    .map_err(|e| Error::ParseJsonError(CONFIG.to_owned(), e))?;

	let conn = Connection::from_file(crate::ENV_FILE)?;
    let me = Arc::new(accounts::verify_credentials::get(&conn).send()?);
    let (tx, rx) = mpsc::channel();

    let tx_for_local = mpsc::Sender::clone(&tx);
    let me_for_local = Arc::clone(&me);
    thread::spawn(move || {
        info!("Start to listen local timeline");

        let listener = LocalTimelineListener::new(&me_for_local, tx_for_local);
        if let Err(e) = listen(StreamType::PublicLocal, &listener) {
            error!("The thread for listening to the public local timeline is dead: {}", e);
            process::exit(1);
        }
    });

    let conn_for_user = Connection::from_file(crate::ENV_FILE)?;
    let tx_for_user = mpsc::Sender::clone(&tx);
    let me_for_user = Arc::clone(&me);
    let follow_config = config.follows.clone();
    thread::spawn(move || {
        info!("Start to listen user timeline");

        let listener = UserTimelineListener::new(
            &conn_for_user,
            &me_for_user,
            tx_for_user,
            &follow_config.follow_regex,
            &follow_config.unfollow_regex,
        );
        if let Err(e) = listen(StreamType::User, &listener) {
            error!("The thread for listening to the user timeline is dead: {}", e);
            process::exit(1);
        }
    });

    let monsters = Monsters::load()?;
    let jashin = contents::Jashin::load(&monsters)?;
    let seishugosha = contents::Seishugosha::load(&monsters)?;
    let boueigun = contents::Boueigun::load(&monsters)?;
    let keema = contents::Keema::load()?;

    let reactions: Vec<&dyn Reaction> = vec![
        &jashin,
        &seishugosha,
        &boueigun,
        &monsters,
    ];

    let mut emojis = Emojis::load(&conn)?;

    for status in rx {
        let content = match status.content() {
            Some(content) => content,
            None => continue,
        };
        trace!("Status received: {:?}", status);

        let response: Option<String>;

        if config.oshiete.keemasan_regex.is_match(content) && config.oshiete.oshiete_regex.is_match(content) {
            trace!("Match keywords for OSHIETE: {}", content);

            let mut r = reactions.iter()
                .map(|i| i.reaction(&ReactionCriteria::new(chrono::Local::now(), content)))
                .filter(|i| i.is_some())
                .map(|i| i.unwrap())
                .collect::<Vec<String>>()
                .join("\n");

            if r.is_empty() {
                r = String::from("？");
            }

            response = Some(r);
        } else {
            trace!("Not match keywords for OSHIETE: {}", content);
            response = keema.reaction(&ReactionCriteria::new(chrono::Local::now(), content));
        }
        trace!("Reaction created: {:?}", response);

        if let Some(response) = response {
            let response = emojis.emojify(
                String::from("@") + status.account().acct() + "\n\n" + response.as_str()
            );
    
            let mut post = statuses::post(&conn, response);
            if status.account().is_remote() {
                post = post.in_reply_to_id(status.id());
            }
    
            match post.send() {
                Ok(posted) => info!(
                    "Reaction completed: status: {:?}: mention: {}",
                    posted.status().unwrap().content(),
                    status.account().acct(),
                ),
                Err(e) => error!("Can't send reply to {}: {}", status.account().acct(), e),
            };
        }
    }

    Ok(())
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
                    info!("Timeline listening recovered: retry: {}", retry);
                }
            },
            Err(e) => {
                retry += 1;
                warn!("Timeline listener returns an error: {}, timeline: {}, retry: {}", e, stream_type, retry);
            }
        };
    }
    Err(Error::LostStreamingConnectionError(stream_type, MAX_RETRY))
}

#[derive(Debug, Clone, Deserialize)]
struct Config {
    oshiete: OshieteConfig,
    follows: FollowConfig,
}

#[derive(Debug, Clone, Deserialize)]
struct OshieteConfig {
    #[serde(deserialize_with = "transform_string_to_regex")]
    keemasan_regex: regex::Regex,
    #[serde(deserialize_with = "transform_string_to_regex")]
    oshiete_regex: regex::Regex,
}

#[derive(Debug, Clone, Deserialize)]
struct FollowConfig {
    #[serde(deserialize_with = "transform_string_to_regex")]
    follow_regex: regex::Regex,
    #[serde(deserialize_with = "transform_string_to_regex")]
    unfollow_regex: regex::Regex,
}

