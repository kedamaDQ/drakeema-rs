pub(crate) mod boueigun;
//pub(crate) mod feed;
pub(crate) mod jashin;
pub(crate) mod keema;
pub(crate) mod monthly_contents;
pub(crate) mod periodic_contents;
pub(crate) mod seishugosha;
pub(crate) mod weekly_activity;
pub(crate) mod weekly_contents;

pub use boueigun::Boueigun;
//pub use feed::Feeds;
pub use jashin::Jashin;
pub use keema::Keema;
pub use monthly_contents::MonthlyContents;
pub use periodic_contents::PeriodicContents;
pub use seishugosha::Seishugosha;
pub use weekly_activity::WeeklyActivity;
pub use weekly_contents::WeeklyContents;

use chrono::{ DateTime, Local };

pub trait Announcer: Sync + Send {
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

pub trait Responder: Sync + Send {
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
