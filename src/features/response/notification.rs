use std::fs::File;
use std::io::BufReader;
use std::sync::mpsc;
use mastors::entities::Notification;
use serde::Deserialize;
use crate::{ Error, Message, Result };
use crate::utils::transform_string_to_regex;

const DATA: &str = "drakeema-data/features/response/notification.json";

#[derive(Debug, Clone)]
pub struct NotificationProcessor {
	config: NotificationConfig,
}

impl NotificationProcessor {
	pub fn load() -> Result<Self> {
		info!("Initialize NotificationProcessor");

		let config: NotificationConfig = serde_json::from_reader(
			BufReader::new(File::open(DATA)?)
		)
		.map_err(|e| Error::ParseJsonError(DATA.to_owned(), e))?;


		Ok(NotificationProcessor {
			config,
		})
	}

	pub fn process(&self, tx: &mpsc::Sender<Message>, notification: &Notification) {
		if notification.is_mention() {
			let status = match notification.status() {
				Some(status) => status,
				None => return,
			};
			let content = match status.content() {
				Some(content) => content,
				None => return,
			};

			if self.config.follow_regex.is_match(content) {
				tx.send(Message::Follow(notification.account().clone())).unwrap();
				tx.send(Message::Status{
					text: self.config.followed_message.to_owned(),
					mention: Some(notification.account().acct().to_owned()),
					visibility: status.visibility(),
					in_reply_to_id: Some(status.id().to_owned()),
				}).unwrap();
			} else if self.config.unfollow_regex.is_match(content) {
				tx.send(Message::Unfollow(notification.account().clone())).unwrap();
				tx.send(Message::Status{
					text: self.config.unfollowed_message.to_owned(),
					mention: Some(notification.account().acct().to_owned()),
					visibility: status.visibility(),
					in_reply_to_id: Some(status.id().to_owned()),
				}).unwrap();
			}
		}
	}
}

#[derive(Debug, Clone, Deserialize)]
struct NotificationConfig {
    #[serde(deserialize_with = "transform_string_to_regex")]
    follow_regex: regex::Regex,
    #[serde(deserialize_with = "transform_string_to_regex")]
	unfollow_regex: regex::Regex,
	
	followed_message: String,
	unfollowed_message: String,
}

