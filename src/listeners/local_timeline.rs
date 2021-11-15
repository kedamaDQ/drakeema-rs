use std::sync::Arc;
use std::sync::mpsc::Sender;
use mastors::prelude::*;
use super::{
	utils,
	TimelineMessage,
};

pub struct LocalTimelineListener {
	me: Arc<Account>,
	tx: Sender<TimelineMessage>,
}

impl LocalTimelineListener {
	pub fn new(me: Arc<Account>, tx: Sender<TimelineMessage>) -> Self {
		LocalTimelineListener {
			me,
			tx,
		}
	}
}

impl EventListener for LocalTimelineListener {
	type Error = crate::Error;

	fn update(&self, status: &Status) -> Result<(), Self::Error> {
		debug!("Receive update: {:?}", status);

		if utils::is_mine(status, &self.me) {
			debug!("Skip update: Status posted by myself: {}", status.id());
			return Ok(());
		}

		if utils::is_mention_to_myself(status, &self.me) {
			debug!("Skip update: Status mentioned to myself: {}", status.id());
			return Ok(())
		}

		if utils::has_spoiler_text(status) {
			debug!("Skip update: Status has spoiler text: {}", status.id());
			return Ok(())
		}

		self.tx.send(TimelineMessage::Status(status.clone())).unwrap();
		Ok(())
	}
}
