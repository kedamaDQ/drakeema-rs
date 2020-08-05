mod jashin;
mod seishugosha;

pub use jashin::Jashin;
pub use seishugosha::Seishugosha;

use chrono::{ DateTime, Local };

pub trait Announcement {
	fn announcement(&self, at: DateTime<Local>) -> String;
}

pub trait Information {
	fn information(&self, at: DateTime<Local>, text: impl AsRef<str>) -> Option<String>;
}
