use std::sync::mpsc::Sender;
use mastors::prelude::*;
use crate::Result;

pub struct LocalTimelineListener<'a> {
	me: &'a Account,
	tx: Sender<Result<Status>>,
}

impl<'a> LocalTimelineListener<'a> {
	pub fn new(me: &'a Account, tx: Sender<Result<Status>>) -> Self {
		LocalTimelineListener {
			me,
			tx,
		}
	}
}

impl<'a> EventListener for LocalTimelineListener<'a> {
	type Error = crate::Error;

	fn update(&self, status: &Status) -> std::result::Result<(), Self::Error> {
		if status.account() == self.me {
			info!("Skip update: Status posted by myself: {}", status.id());
			return Ok(());
		}
		self.tx.send(Ok(status.clone())).map_err(|e| crate::Error::SendStatusMessageError(Box::new(e)))
	}
}
