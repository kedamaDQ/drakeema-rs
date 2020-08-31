use std::sync::mpsc;
use std::thread;
use mastors::{
	Connection,
	api::v1::statuses,
};
use crate::{
	Emojis,
	Monsters,
	Result,
};
use crate::contents::{
	Jashin,
	MonthlyContents,
	PeriodicContents,
	Seishugosha,
	WeeklyActivity,
	WeeklyContents,
};
use super::{
	Announcer,
	announcement_contents::ContentsAnnouncement,
	announcement_feeds::FeedsAnnouncement,
};

const INTERVAL_SECS: u64 = 600;

pub fn start() -> Result<()> {
	let conn = Connection::new()?;
	let (tx, rx) = mpsc::channel();

	let tx_for_feed = mpsc::Sender::clone(&tx);
	let feeds = thread::spawn(move || {
		FeedsAnnouncement::load(tx_for_feed)
			.expect("Failed to load FeedsAnnouncement")
			.process();
	});

	let monsters = Monsters::load()?;
	let periodic_contents = PeriodicContents::load()?;
	let weekly_contents = WeeklyContents::load()?;
	let monthly_contents = MonthlyContents::load()?;
	let seishugosha = Seishugosha::load(&monsters)?;
	let jashin = Jashin::load(&monsters)?;
	let weekly_activity = WeeklyActivity::load()?;

	let tx_for_contents = mpsc::Sender::clone(&tx);
	let contents = thread::spawn(move || {
		let contents: Vec<&dyn Announcer> = vec![
			&periodic_contents,
			&weekly_contents,
			&monthly_contents,
			&seishugosha,
			&jashin,
			&weekly_activity,
		];

		ContentsAnnouncement::load(contents, tx_for_contents)
			.expect("Failed to load ContentsAnnouncement")
			.process();
	});

	let mut emojis = Emojis::load(&conn)?;
	for text in rx {
		match statuses::post(&conn, emojis.emojify(text)).send() {
			Ok(_) => info!("Completed to announcement"),
			Err(e) => error!("Failed to announcement: {}", e),
		}
	}

	feeds.join().unwrap();
	contents.join().unwrap();
	Ok(())
}
