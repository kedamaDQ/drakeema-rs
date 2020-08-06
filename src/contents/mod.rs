mod jashin;
mod seishugosha;

pub use jashin::Jashin;
pub use seishugosha::Seishugosha;

use chrono::{ DateTime, Local };

pub trait Announcement {
	fn announcement(&self, criteria: AnnouncementCriteria) -> String;
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
}

pub trait Information {
	fn information(&self, criteria: InformationCriteria) -> Option<String>;
}

pub struct InformationCriteria {
	at: DateTime<Local>,
	text: String,
}

impl InformationCriteria {
	pub fn new(at: DateTime<Local>, text: impl Into<String>) -> Self {
		InformationCriteria {
			at,
			text: text.into(),
		}
	}
}
