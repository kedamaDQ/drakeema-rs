mod boueigun;
mod jashin;
mod seishugosha;

pub use jashin::Jashin;
pub use seishugosha::Seishugosha;
pub use boueigun::Boueigun;

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

pub trait Reaction {
	fn reaction(&self, criteria: ReactionCriteria) -> Option<String>;
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
}
