use std::process;
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;
use mastors::prelude::*;
use mastors::{
	Connection,
	api::v1::accounts,
	api::v1::streaming,
};
use crate::{
    Error,
    Monsters,
    Result,
    contents,
	listeners::{
        LocalTimelineListener,
        UserTimelineListener,
        Message,
    },
};
use super::{
    Responder,
    response_status,
    response_notification,
};

const MAX_RETRY: usize = 5;

pub fn attach() -> Result<()> {
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

    let tx_for_user = mpsc::Sender::clone(&tx);
    let me_for_user = Arc::clone(&me);
    thread::spawn(move || {
        info!("Start to listen user timeline");

        let listener = UserTimelineListener::new(
            &me_for_user,
            tx_for_user,
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

	let responders: Vec<&dyn Responder> = vec![
        &jashin,
       	&seishugosha,
       	&boueigun,
       	&monsters,
	];

    let keema = contents::Keema::load()?;

    let mut response_status = response_status::Response::load(&conn, responders, &keema)?;
    let mut response_notification = response_notification::Response::load(&conn)?;

    for message in rx {
        match message {
            Message::Status(status) => &response_status.process(&status),
            Message::Notification(notification) => &response_notification.process(&notification),
        };
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
