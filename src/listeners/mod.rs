mod user_timeline;
mod local_timeline;
pub(crate) mod utils;

pub use user_timeline::UserTimelineListener;
pub use local_timeline::LocalTimelineListener;

use mastors::entities::{
	Status,
	Notification,
};

#[derive(Debug, Clone)]
pub enum TimelineMessage {
	Status(Status),
	Notification(Notification),
}

