use chrono::Local;
use mastors::{
	Connection,
	api::v1::statuses,
};
use crate::{
	Emojis,
	Monsters,
	Result
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
	Announcement,
	AnnouncementCriteria,
};

pub fn announce() -> Result<()> {
	let monsters = Monsters::load()?;
	let periodic_contents = PeriodicContents::load()?;
	let weekly_contents = WeeklyContents::load()?;
	let monthly_contents = MonthlyContents::load()?;
	let seishugosha = Seishugosha::load(&monsters)?;
	let jashin = Jashin::load(&monsters)?;
	let weekly_activity = WeeklyActivity::load()?;
	let contents: Vec<&dyn Announcement> = vec![
		&periodic_contents,
		&weekly_contents,
		&monthly_contents,
		&seishugosha,
		&jashin,
		&weekly_activity,
	];
	let criteria = AnnouncementCriteria::new(Local::now());

	let text = contents.iter()
		.map(|c| c.announcement(&criteria))
		.filter(|c| c.is_some())
		.map(|c| c.unwrap())
		.collect::<Vec<String>>()
		.join("\n\n");

	if !text.is_empty() {
		let conn = Connection::from_file(crate::ENV_FILE)?;
		statuses::post(
			&conn,
			Emojis::load(&conn)?.emojify(text)
		).send()?;
	}

	Ok(())
}
