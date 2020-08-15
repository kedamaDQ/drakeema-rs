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
	Seishugosha,
};
use super::{
	Announcement,
	AnnouncementCriteria,
};

pub fn announce() -> Result<()> {
	let monsters = Monsters::load()?;
	let jashin = Jashin::load(&monsters)?;
	let seishugosha = Seishugosha::load(&monsters)?;
	let contents: Vec<&dyn Announcement> = vec![&seishugosha, &jashin];
	let criteria = AnnouncementCriteria::new(Local::now());

	let text = contents.iter()
		.map(|c| c.announcement(&criteria))
		.filter(|c| c.is_some())
		.map(|c| c.unwrap())
		.collect::<Vec<String>>()
		.join("\n\n");

	if !text.is_empty() {
		let conn = Connection::from_file(".env.test.st")?;
		statuses::post(
			&conn,
			Emojis::load(&conn)?.emojify(text)
		).send()?;
	}

	Ok(())
}

struct AnnouncementJson {

}
