pub(crate) mod announcement;
mod announcement_contents;
mod announcement_feeds;
pub(crate) mod response;
pub(crate) mod rate_limit;
mod response_status;
mod response_notification;

use chrono::{ DateTime, Local };

pub trait Announcer {
	fn announce(&self, criteria: &AnnouncementCriteria) -> Option<String>;
}

#[derive(Debug, Clone)]
pub struct AnnouncementCriteria {
	at: DateTime<Local>,
}

impl AnnouncementCriteria {
	pub fn new(at: DateTime<Local>) -> Self {
		AnnouncementCriteria {
			at,
		}
	}

	pub fn at(&self) -> DateTime<Local> {
		self.at
	}
}

pub trait Responder {
	fn respond(&self, criteria: &ResponseCriteria) -> Option<String>;
}

#[derive(Debug, Clone)]
pub struct ResponseCriteria {
	at: DateTime<Local>,
	text: String,
}

impl ResponseCriteria {
	pub fn new(at: DateTime<Local>, text: impl Into<String>) -> Self {
		ResponseCriteria {
			at,
			text: text.into(),
		}
	}

	pub fn at(&self) -> DateTime<Local> {
		self.at
	}

	pub fn text(&self) -> &str {
		&self.text
	}
}
