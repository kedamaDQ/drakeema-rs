use std::sync::mpsc::Sender;
use mastors::prelude::*;
use crate::Error;
use super::Message;

pub struct UserTimelineListener<'a> {
	me: &'a Account, 
	tx: Sender<Message>,
}

impl<'a> UserTimelineListener<'a> {
	pub fn new(
		me: &'a Account,
		tx: Sender<Message>,
	) -> Self {
		UserTimelineListener {
			me,
			tx,
		}
	}
}

impl<'a> EventListener for UserTimelineListener<'a> {
	type Error = crate::Error;

	fn update(&self, status: &Status) -> Result<(), Self::Error> {
		trace!("Receive update: {:?}", status);

		if status.account() == self.me {
			info!("Skip update: Status posted by myself: {}", status.id());
			return Ok(());
		}

		if is_overlapped_at_local_and_home(status) {
			trace!("Skip update: Status overlapped at local and home: {}", status.id());
			return Ok(());
		}
		self.tx.send(Message::Status(status.clone())).map_err(|e| Error::SendMessageForResponseError(Box::new(e)))
	}

	fn notification(&self, notification: &Notification) -> Result<(), Self::Error> {
		info!("Notification raceived: {}: {}", notification.notification_type(), notification.id());
		self.tx.send(Message::Notification(notification.clone())).map_err(|e| Error::SendMessageForResponseError(Box::new(e)))
	}
}

fn is_overlapped_at_local_and_home(status: &Status) -> bool {
    status.account().is_local() && status.visibility() == Visibility::Public
}
