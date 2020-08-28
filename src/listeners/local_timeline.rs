use std::sync::mpsc::Sender;
use mastors::prelude::*;
use super::Message;

pub struct LocalTimelineListener<'a> {
	me: &'a Account,
	tx: Sender<Message>,
}

impl<'a> LocalTimelineListener<'a> {
	pub fn new(me: &'a Account, tx: Sender<Message>) -> Self {
		LocalTimelineListener {
			me,
			tx,
		}
	}
}

impl<'a> EventListener for LocalTimelineListener<'a> {
	type Error = crate::Error;

	fn update(&self, status: &Status) -> Result<(), Self::Error> {
		trace!("Receive update: {:?}", status);

		if status.account() == self.me {
			info!("Skip update: Status posted by myself: {}", status.id());
			return Ok(());
		}

		if status.mentions().iter().any(|m| m.acct() == self.me.acct()) {
			info!("Skip update: Status mentioned to myself: {}", status.id());
			return Ok(())
		}

		self.tx.send(Message::Status(status.clone())).map_err(|e| crate::Error::SendMessageError(Box::new(e)))
	}
}
