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
	Announcer,
	AnnouncementCriteria,
};

pub fn announce(criteria: &AnnouncementCriteria) -> Result<()> {
	let monsters = Monsters::load()?;
	let periodic_contents = PeriodicContents::load()?;
	let weekly_contents = WeeklyContents::load()?;
	let monthly_contents = MonthlyContents::load()?;
	let seishugosha = Seishugosha::load(&monsters)?;
	let jashin = Jashin::load(&monsters)?;
	let weekly_activity = WeeklyActivity::load()?;
	let contents: Vec<&dyn Announcer> = vec![
		&periodic_contents,
		&weekly_contents,
		&monthly_contents,
		&seishugosha,
		&jashin,
		&weekly_activity,
	];
	info!("Start announcement: {:?}", criteria);

	let text = contents.iter()
		.map(|c| c.announce(&criteria))
		.filter(|c| c.is_some())
		.map(|c| c.unwrap())
		.collect::<Vec<String>>()
		.join("\n\n");

	if !text.is_empty() {
		let date_string = criteria.at.format("[%Y-%m-%d]").to_string();
		let conn = Connection::from_file(crate::ENV_FILE)?;
		let announcement = format!("{}\n\n{}", date_string, Emojis::load(&conn)?.emojify(text));
		info!("Found announcement: {}", announcement);

		statuses::post(
			&conn,
			announcement,
		).send()?;
	}

	Ok(())
}
