mod notification;
mod status;

use std::sync::Arc;
use std::sync::mpsc;
use std::thread;
use chrono::{ Duration, Local };
use mastors::prelude::*;
use mastors::{
	api::v1::accounts,
	api::v1::streaming,
};
use crate::{ Error, Message, Result, };
use crate::listeners::{
	LocalTimelineListener,
	UserTimelineListener,
	TimelineMessage,
};
use notification::NotificationProcessor;
use status::StatusProcessor;

const MAX_RETRY: usize = 5;
const RETRY_RESET_INTERVAL_SECS: i64 = 300;

pub struct ResponseWorker {
	me: Arc<Account>,
	notification_processor: Arc<NotificationProcessor>,
	status_processor: Arc<StatusProcessor>,
}

impl ResponseWorker {
	pub fn load() -> Result<Self> {
		info!("Initialize ResponseWorker");

		let conn = Connection::new()?;

		Ok(ResponseWorker {
			me: Arc::new(accounts::verify_credentials::get(&conn).send()?),
			notification_processor: Arc::new(NotificationProcessor::load()?),
			status_processor: Arc::new(StatusProcessor::load()?),
		})
	}

	pub fn start(&self, tx: mpsc::Sender<Message>) {
		let me = Arc::clone(&self.me);
		let notification_processor = Arc::clone(&self.notification_processor);
		let status_processor = Arc::clone(&self.status_processor);

		let outer_tx_for_local = mpsc::Sender::clone(&tx);
		let outer_tx_for_user = mpsc::Sender::clone(&tx);
		thread::spawn(move || {
			let (inner_tx, inner_rx) = mpsc::channel();

			let me_for_local = Arc::clone(&me);
			let tx_for_local = mpsc::Sender::clone(&inner_tx);
			thread::spawn(move || {
				let listener = LocalTimelineListener::new(me_for_local, tx_for_local);
				if let Err(e) = listen(StreamType::PublicLocal, &listener) {
					outer_tx_for_local.send(Message::Error("Failed to connect to local timeline".to_owned(), e)).unwrap();
				}
			});

			let me_for_user = Arc::clone(&me);
			let tx_for_user = mpsc::Sender::clone(&inner_tx);
			thread::spawn(move || {
				let listener = UserTimelineListener::new(me_for_user, tx_for_user);
				if let Err(e) = listen(StreamType::User, &listener) {
					outer_tx_for_user.send(Message::Error("Failed to connect to user timeline".to_owned(), e)).unwrap();
				}
			});

			let tx_for_loop = mpsc::Sender::clone(&tx);
			for timeline_message in inner_rx {
				match timeline_message {
					TimelineMessage::Notification(notification) => {
						notification_processor.process(&tx_for_loop, &notification);
					},
					TimelineMessage::Status(status) => {
						status_processor.process(&tx_for_loop, &status);
					},
				};
			}
		});

	}
}

fn listen(
	stream_type: StreamType,
	listener: &impl EventListener,
) -> Result<()> {
	let conn = Connection::new()?;
	let mut stream = streaming::get(&conn, stream_type.clone()).send()?;
	let mut retry = 0;
	let mut last_retry = Local::now();
	while retry < MAX_RETRY {
		if let Err(e) = stream.attach(listener) {
			if Local::now() - last_retry > Duration::seconds(RETRY_RESET_INTERVAL_SECS) {
				retry = 1;
			} else {
				retry += 1;
			}
			warn!(
				"Timeline listener returns an error: {}, timeline: {}, last_retry: {}, count: {}",
				e, stream_type, last_retry, retry
			);
			last_retry = Local::now();
		};
	}
	Err(Error::LostStreamingConnectionError(stream_type, MAX_RETRY))
}
