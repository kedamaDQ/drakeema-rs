use mastors::prelude::*;
use mastors::api::v1::{
	statuses,
	accounts::id::{ follow, unfollow },
};
use crate::Result;
use crate::emojis::Emojis;
use crate::rate_limit::RateLimit;

pub struct MessageProcessor<'a> {
	conn: &'a Connection,
	emojis: Emojis<'a>,
	limit_for_status: RateLimit,
	limit_for_ff: RateLimit,
}

impl<'a> MessageProcessor<'a> {
	pub fn new(conn: &'a Connection) -> Result<Self> {
		info!("Initialize MessageProcessor");

		Ok(MessageProcessor {
			conn,
			emojis: Emojis::load(conn)?,
			limit_for_status: RateLimit::new(20),
			limit_for_ff: RateLimit::new(10),
		})
	}

	pub fn process(&mut self, msg: Message) -> Result<()> {
		match msg {
			Message::Status{
				text, visibility, mention, in_reply_to_id, poll_options,
			} => self.status(text, visibility, mention, in_reply_to_id, poll_options),
			Message::Follow(id) => self.follow(id),
			Message::Unfollow(id) => self.unfollow(id),
			Message::Error(text, e) => {
				error!("Received error message: {}: {}", text, e);
				Err(e)
			}
		}
	}

	fn status(
		&mut self,
		text: String,
		visibility: Visibility,
		mention: Option<String>,
		in_reply_to_id: Option<String>,
		poll_options: Option<PollOptions>,
	) -> Result<()> {
		info!("Start posting a Status");

		if let Err(e) = self.limit_for_status.increment() {
			return Err(e);
		}
		info!("Rate limit status for Status: {}", self.limit_for_status);

		let text = match mention {
			Some(mention) => format!("@{} {}", mention, text),
			None => text,
		};

		let post = statuses::post(self.conn).status(self.emojis.emojify(text));
		let post = match in_reply_to_id {
			Some(id) => post.in_reply_to_id(id),
			None => post,
		};

		let result = match poll_options {
			Some(po) => post
				.visibility(visibility)
				.poll(po.poll_options, po.expires_in)
				.send(),
			None => post
				.visibility(visibility)
				.send(),
		};
		match result {
			Ok(posted) => info!(
				"Posting a status is complete: {}",
				posted.content().unwrap().replace("\n", "")
			),
			Err(e) => error!("Failed to post status: {}", e),
		};

		Ok(())
	}

	fn follow(&mut self, account: Account) -> Result<()> {
		info!("Start following an account: {}", account.acct());

		if let Err(e) = self.limit_for_ff.increment() {
			return Err(e);
		}
		info!("Rate limit status for follow/unfollow: {}", self.limit_for_ff);

		match follow::post(self.conn, account.id()).send() {
			Ok(_) => info!("Following an account is complete: {}", account.acct()),
			Err(e) => error!("Failed to follow an account: {}, error: {}", account.acct(), e)
		};

		Ok(())
	}

	fn unfollow(&mut self, account: Account) -> Result<()> {
		info!("Start unfollowing an account: {}", account.acct());

		if let Err(e) = self.limit_for_ff.increment() {
			return Err(e);
		}
		info!("Rate limit status for follow/unfollow: {}", self.limit_for_ff);

		match unfollow::post(self.conn, account.id()).send() {
			Ok(_) => info!("Unfollowing an account is complete: {}", account.acct()),
			Err(e) => error!("Failed to unfollow an account: {}, error: {}", account.acct(), e),
		}

		Ok(())
	}

}

#[derive(Debug)]
pub enum Message {
	Status {
		text: String,
		visibility: Visibility,
		mention: Option<String>,
		in_reply_to_id: Option<String>,
		poll_options: Option<PollOptions>,
	},
	Follow(Account),
	Unfollow(Account),
	Error(String, crate::Error),
}

#[derive(Debug, Clone)]
pub struct PollOptions {
	poll_options: Vec<String>,
	expires_in: u64,
}

impl PollOptions {
	pub fn new(poll_options: Vec<String>, expires_in: u64) -> Self {
		PollOptions {
			poll_options,
			expires_in,
		}
	}
}
