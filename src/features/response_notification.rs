use std::fs::File;
use std::io::BufReader;
use mastors::{
	Connection,
	Method,
	api::v1::accounts::id::{ follow, unfollow },
	entities::Notification,
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
				Some(status) => status.content().unwrap_or(""),
				None => "",
			};
			if self.config.follow_regex.is_match(status) {
				if let Err(e) = self.rate_limit.increment() {
					error!("Follow/Unfollow ratelimit exceeded: {}", e);
					return Err(e);
				}

				info!("Follow an account: {}", notification.account().id());
				match follow::post(
					self.conn, notification.account().id()
				).send() {
					Ok(_) => info!("Follow an account: {}", notification.account().acct()),
					Err(e) => error!("Failed to follow an account: account: {}, error: {}", notification.account().acct(), e),
				};
			} else if self.config.unfollow_regex.is_match(status) {
				if let Err(e) = self.rate_limit.increment() {
					error!("Follow/Unfollow ratelimit exceeded: {}", e);
					return Err(e);
				}

				info!("Unfollow an account: {}", notification.account().id());
				match unfollow::post(
					self.conn, notification.account().id()
				).send() {
					Ok(_) => info!("Follow an account: {}", notification.account().acct()),
					Err(e) => error!("Failed to follow an account: account: {}, error: {}", notification.account().acct(), e),
				};
			}
		}

		Ok(())
	}
}

#[derive(Debug, Clone, Deserialize)]
struct NotificationConfig {
    #[serde(deserialize_with = "transform_string_to_regex")]
    follow_regex: regex::Regex,
    #[serde(deserialize_with = "transform_string_to_regex")]
	unfollow_regex: regex::Regex,
	
	rate_limit: usize,
}

