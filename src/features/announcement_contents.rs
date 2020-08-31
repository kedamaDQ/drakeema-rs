use std::fs::File;
use std::io::BufReader;
use std::sync::mpsc;
use std::thread;
use chrono::{ DateTime, Duration, Local, NaiveTime };
use serde::Deserialize;
use crate::{ Error, Result };
use super::{ Announcer, AnnouncementCriteria, };

const DATA: &str = "drakeema-data/announcement_contents.json";

pub struct ContentsAnnouncement<'a> {
	contents: Vec<&'a dyn Announcer>,
	announcement_time: NaiveTime,
	tx: mpsc::Sender<String>,
}

impl<'a> ContentsAnnouncement<'a> {
	pub fn load(contents: Vec<&'a dyn Announcer>, tx: mpsc::Sender<String>) -> Result<Self> {
		let json: ContentsAnnouncementJson = serde_json::from_reader(
			BufReader::new(File::open(DATA)?)
		)
		.map_err(|e| Error::ParseJsonError(DATA.to_owned(), e))?;

		Ok(ContentsAnnouncement {
			contents,
			announcement_time: json.announcement_time,
			tx,
		})
	}

	pub fn process(&mut self) {
		loop {
			let now = Local::now();
			thread::sleep(
				// to_std() returns error if duration is negative.
				// next_announce() never return negative duration.
				(self.next_announce(now) - Local::now()).to_std().unwrap()
			);

			let criteria = AnnouncementCriteria::new(Local::now());
			info!("Start announcement: {:?}", criteria);

			let text = self.contents.iter()
				.map(|c| c.announce(&criteria))
				.filter(|c| c.is_some())
				.map(|c| c.unwrap())
				.collect::<Vec<String>>()
				.join("\n\n");

			if !text.is_empty() {
				self.tx.send(text).expect("Failed to end message from ContentsAnnouncement");
			}
		}
	}

	fn next_announce(&self, now: DateTime<Local>) -> DateTime<Local> {
		let base_date = now.date().and_time(self.announcement_time).expect("Failed to create base date time");
		if now >= base_date && now < base_date + Duration::hours(12) {
			base_date + Duration::hours(12)
		} else {
			base_date + Duration::hours(24)
		}
	}
}

#[derive(Debug, Clone, Deserialize)]
struct ContentsAnnouncementJson {
	announcement_time: NaiveTime,
}
