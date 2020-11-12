use std::sync::Arc;
use std::sync::mpsc;
use mastors::prelude::*;
use super::TimelineMessage;

pub struct UserTimelineListener {
	me: Arc<Account>, 
	tx: mpsc::Sender<TimelineMessage>,
}

impl UserTimelineListener {
	pub fn new(
		me: Arc<Account>,
		tx: mpsc::Sender<TimelineMessage>,
	) -> Self {
		UserTimelineListener {
			me,
			tx,
		}
	}
}

impl EventListener for UserTimelineListener {
	type Error = crate::Error;

	fn update(&self, status: &Status) -> Result<(), Self::Error> {
		debug!("Receive update: {:?}", status);

		if status.account().id() == self.me.id() {
			debug!("Skip update: Status posted by myself: {}", status.id());
			return Ok(());
		}

		if is_overlapped_at_local_and_home(status) {
			debug!("Skip update: Status overlapped at local and home: {}", status.id());
			return Ok(());
		}

		if is_boosted(status) {
			debug!("Skip update: Status is boosted status");
			return Ok(());
		}
		self.tx.send(TimelineMessage::Status(status.clone())).unwrap();
		Ok(())
	}

	fn notification(&self, notification: &Notification) -> Result<(), Self::Error> {
		debug!("Notification raceived: {}: {}", notification.notification_type(), notification.id());
		self.tx.send(TimelineMessage::Notification(notification.clone())).unwrap();
		Ok(())
	}
}

fn is_overlapped_at_local_and_home(status: &Status) -> bool {
    status.account().is_local() && status.visibility() == Visibility::Public
}

fn is_boosted(status: &Status) -> bool {
	status.reblog().is_some()
}
