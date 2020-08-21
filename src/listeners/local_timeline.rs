use std::sync::mpsc::Sender;
use mastors::prelude::*;

pub struct LocalTimelineListener<'a> {
	me: &'a Account,
	tx: Sender<Status>,
}

impl<'a> LocalTimelineListener<'a> {
	pub fn new(me: &'a Account, tx: Sender<Status>) -> Self {
		LocalTimelineListener {
			me,
			tx,
		}
	}
}

impl<'a> EventListener for LocalTimelineListener<'a> {
	type Error = crate::Error;

	fn update(&self, status: &Status) -> Result<(), Self::Error> {
		if status.account() == self.me {
			info!("Skip update: Status posted by myself: {}", status.id());
			return Ok(());
		}
		self.tx.send(status.clone()).map_err(|e| crate::Error::SendStatusMessageError(Box::new(e)))
	}
}
