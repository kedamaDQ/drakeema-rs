use mastors::prelude::*;
use mastors::api::v1::accounts::id::{
	follow,
	unfollow,
};
use regex::Regex;

pub struct UserTimelineListener<'a> {
	conn: &'a Connection,
	me: &'a Account, 
}

impl<'a> UserTimelineListener<'a> {
	pub fn new(conn: &'a Connection, me: &'a Account) -> Self {
		UserTimelineListener {
			conn,
			me,
		}
	}
}

const REGEX_FOLLOW_REQUEST: &str = r#"(?:フォロー|ふぉろー|follow)\s*して"#;
const REGEX_UNFOLLOW_REQUEST: &str = r#"(?:フォロー|ふぉろー|follow)\s*(?:やめて|はずして|外して)"#;

impl<'a> EventListener for UserTimelineListener<'a> {
	type Error = mastors::Error;

	fn update(&self, status: &Status) -> Result<(), Self::Error> {
		if status.account() == self.me {
			info!("Skip update: Status posted by myself: {}", status.id());
			return Ok(());
		}

		if is_overlapped_at_local_and_home(status) {
			info!("Skip update: Status overlapped at local and home: {}", status.id());
			return Ok(());
		}

//		
		Ok(())
	}

	fn notification(&self, notification: &Notification) -> Result<(), Self::Error> {
		info!("Notification raceived: {}: {}", notification.notification_type(), notification.id());

		if notification.is_mention() {
			if is_match(REGEX_FOLLOW_REQUEST, notification.status()) {
				info!("Follow an account: {}", notification.account().id());
				follow::post(
					self.conn, notification.account().id()
				).send()?;
			} else if is_match(REGEX_UNFOLLOW_REQUEST, notification.status()) {
				info!("Unfollow an account: {}", notification.account().id());
				unfollow::post(
					self.conn, notification.account().id()
				).send()?;
			}
		} else {
			info!("Skip notification: {}: {}", notification.notification_type(), notification.id());
		}

		Ok(())
	}
}

fn is_overlapped_at_local_and_home(status: &Status) -> bool {
    status.account().is_local() && status.visibility() == Visibility::Public
}

fn is_only_at_home(status: &Status) -> bool {
    !is_overlapped_at_local_and_home(status)
}

fn is_match(regex: &str, status: Option<&Status>) -> bool {
	Regex::new(regex)
		.unwrap()
		.is_match(
			status
				.map(|s| s.content().unwrap_or(""))
				.unwrap_or("")
		)
}
