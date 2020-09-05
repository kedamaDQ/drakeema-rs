use std::sync::Arc;
use std::sync::mpsc::Sender;
use mastors::prelude::*;
use super::TimelineMessage;

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

		if status.account().id() == self.me.id() {
			debug!("Skip update: Status posted by myself: {}", status.id());
			return Ok(());
		}

		if status.mentions().iter().any(|m| m.acct() == self.me.acct()) {
			debug!("Skip update: Status mentioned to myself: {}", status.id());
			return Ok(())
		}

		self.tx.send(TimelineMessage::Status(status.clone())).unwrap();
		Ok(())
	}
}
