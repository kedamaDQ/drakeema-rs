mod announcement;
mod reaction;

pub use announcement::announce;
pub use reaction::attach;

use chrono::{ DateTime, Local };

pub trait Announcement {
	fn announcement(&self, criteria: &AnnouncementCriteria) -> Option<String>;
}

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

pub trait Reaction {
	fn reaction(&self, criteria: &ReactionCriteria) -> Option<String>;
}

pub struct ReactionCriteria {
	at: DateTime<Local>,
	text: String,
}

impl ReactionCriteria {
	pub fn new(at: DateTime<Local>, text: impl Into<String>) -> Self {
		ReactionCriteria {
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
