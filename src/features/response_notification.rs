use std::fs::File;
use std::io::BufReader;
use mastors::{
	Connection,
	Method,
	api::v1::accounts::id::{ follow, unfollow },
	api::v1::statuses,
	entities::{ Notification },
};
use serde::Deserialize;
use crate::{
	Error,
	Result,
	utils::transform_string_to_regex,
};
use super::rate_limit::RateLimit;


const DATA: &str = "drakeema-data/response_notification.json";

pub struct Response<'a> {
	conn: &'a Connection,
	config: NotificationConfig,
	rate_limit: RateLimit,
}

impl<'a> Response<'a> {
	pub fn load(conn: &'a Connection) -> Result<Self> {
		let config: NotificationConfig = serde_json::from_reader(
			BufReader::new(File::open(DATA)?)
		)
		.map_err(|e| Error::ParseJsonError(DATA.to_owned(), e))?;

		let rate_limit = RateLimit::new(config.rate_limit);

		Ok(Response {
			conn,
			config,
			rate_limit,
		})
	}

	pub fn process(&mut self, notification: &Notification) -> Result<()> {
		if notification.is_mention() {
			let status = match notification.status() {
				Some(status) => match status.content() {
					Some(content) => content,
					None => return Ok(()),
				},
				None => return Ok(()),
			};
			if self.config.follow_regex.is_match(status) {
				return self.follow_unfollow(FoUnfo::Follow, &notification);
			} else if self.config.unfollow_regex.is_match(status) {
				return self.follow_unfollow(FoUnfo::Unfollow, &notification);
			}
		}

		Ok(())
	}

	fn follow_unfollow(&mut self, fu: FoUnfo, notification: &Notification) -> Result<()> {
		if let Err(e) = self.rate_limit.increment() {
			error!("Follow/Unfollow rate limit exceeded: {}", e);
			return Err(e);
		}

		let mut response_message = format!("@{}", notification.account().acct());
		let result = match fu {
			FoUnfo::Follow => {
				response_message = format!("{}\n\n{}", response_message ,&self.config.followed_message);
				follow::post(&self.conn, notification.account().id()).send()
			},
			FoUnfo::Unfollow => {
				response_message = format!("{}\n\n{}", response_message ,&self.config.unfollowed_message);
				unfollow::post(&self.conn, notification.account().id()).send()
			},
		};
		
		match result {
			Ok(_) => info!("{} an account: {}", fu, notification.account().acct()),
			Err(e) => {
				error!(
					"Failed to {} an account: account: {}, error: {}",
					fu.to_string().to_lowercase(),
					notification.account().acct(),
					e
				);
				return Ok(())
			}
		}

		match statuses::post(&self.conn, &response_message)
			.in_reply_to_id(notification.status().unwrap().id())
			.send()
		{
			Ok(_) => info!("Send reply: {}", &response_message),
			Err(e) => error!("Failed to send reply: {}, error: {}", &response_message, e),
		};
		Ok(())
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum FoUnfo {
	Follow,
	Unfollow,
}

impl std::fmt::Display for FoUnfo {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			FoUnfo::Follow => write!(f, "Follow"),
			FoUnfo::Unfollow => write!(f, "Unfollow"),
		}
	}
}

#[derive(Debug, Clone, Deserialize)]
struct NotificationConfig {
    #[serde(deserialize_with = "transform_string_to_regex")]
    follow_regex: regex::Regex,
    #[serde(deserialize_with = "transform_string_to_regex")]
	unfollow_regex: regex::Regex,
	
	rate_limit: usize,
	followed_message: String,
	unfollowed_message: String,
}

