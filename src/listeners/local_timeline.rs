use mastors::prelude::*;

pub struct LocalTimelineListener<'a> {
	conn: &'a Connection,
	me: &'a Account,
}

impl<'a> LocalTimelineListener<'a> {
	pub fn new(conn: &'a Connection, me: &'a Account) -> Self {
		LocalTimelineListener {
			conn,
			me,
		}
	}
}

impl<'a> EventListener for LocalTimelineListener<'a> {
	type Error = String;

	fn update(&self, status: &Status) -> Result<(), Self::Error> {
		if status.account() == self.me {
			info!("Skip update: Status posted by myself: {}", status.id());
			return Ok(());
		}
		Ok(())
	}
}
