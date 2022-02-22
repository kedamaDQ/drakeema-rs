use std::fs::File;
use std::io::BufReader;
use chrono::{ DateTime, Duration, Local };
use serde::Deserialize;
use crate::{
	Error,
	Result,
};
use super::{ Announcer, AnnouncementCriteria };

const DATA: &str = "drakeema-data/contents/monthly_contents.json";

#[derive(Debug, Clone, Deserialize)]
pub struct MonthlyContents {
	announcement_at_start: String,
	announcement_at_end: String,
	contents: Vec<Content>,
}

impl MonthlyContents {
	pub fn load() -> Result<Self> {
		info!("Initialize MonthlyContents");

		serde_json::from_reader(
			BufReader::new(File::open(DATA)?)
		)
		.map_err(|e| Error::UnparseableJson(DATA.to_owned(), e))
	}

	fn contents_to_start(&self, at: &DateTime<Local>) -> String {
		self.contents(at, &self.announcement_at_start)
	}

	fn contents_to_end(&self, at: &DateTime<Local>) -> String {
		let tomorrow = *at + Duration::days(1);
		self.contents(&tomorrow, &self.announcement_at_end)
	}

	fn contents(&self, at: &DateTime<Local>, template: &str) -> String {
		use chrono::Datelike;

		let contents = self.contents.iter()
			.filter(|c| c.days.contains(&at.day()))
			.map(|c| c.display.to_owned())
			.collect::<Vec<String>>()
			.join("、");
		
		if contents.is_empty() {
			contents
		} else {
			template.replace("__CONTENTS__", &contents)
		}
	}
}

impl Announcer for MonthlyContents {
	fn announce(&self, criteria: &AnnouncementCriteria) -> Option<String> {
		debug!("Start building announce about MonthlyContents: {:?}", criteria);

		let contents = [
			self.contents_to_end(&criteria.at()),
			self.contents_to_start(&criteria.at()),
		].iter()
		.filter(|c| !c.is_empty())
		.map(|c| c.to_owned())
		.collect::<Vec<String>>()
		.join("\n");

		if contents.is_empty() {
			debug!("Nothing announcement about MonthlyContents: {:?}", criteria);
			None
		} else {
			Some(contents)
		}
	}
}

#[derive(Debug, Clone, Deserialize)]
struct Content {
	#[allow(dead_code)]
	id: String,
	display: String,
	days: Vec<u32>,
}

#[cfg(test)]
pub(crate) mod tests {
	use super::*;
	use chrono::offset::TimeZone;

	#[test]
	fn test_contents_to_start_is_exist() {
		let pc = data();

		let an = pc.announce(
			&AnnouncementCriteria::new(Local.ymd(2020, 8, 1).and_hms(12, 0, 0))
		);
		assert_eq!(an.unwrap(), "今期の :m_nasubimera: シアトリカルクロニクル、不思議の魔塔は今日からです！");

		let an = pc.announce(
			&AnnouncementCriteria::new(Local.ymd(2020, 8, 15).and_hms(12, 0, 0))
		);
		assert_eq!(an.unwrap(), "今期の :m_nasubimera: シアトリカルクロニクルは今日からです！");
	}

	#[test]
	fn test_contents_to_end_is_exist() {
		let pc = data();

		let an = pc.announce(
			&AnnouncementCriteria::new(Local.ymd(2020, 8, 31).and_hms(12, 0, 0))
		);
		assert_eq!(an.unwrap(), "今期の :m_nasubimera: シアトリカルクロニクル、不思議の魔塔は今日までです！");

		let an = pc.announce(
			&AnnouncementCriteria::new(Local.ymd(2020, 8, 14).and_hms(12, 0, 0))
		);
		assert_eq!(an.unwrap(), "今期の :m_nasubimera: シアトリカルクロニクルは今日までです！");
	}

	#[test]
	fn test_contents_is_nothing() {
		let pc = data();

		let an = pc.announce(
			&AnnouncementCriteria::new(Local.ymd(2020, 8, 16).and_hms(12, 0, 0))
		);
		assert!(an.is_none());
	}

	pub(crate) fn data() -> MonthlyContents {
		serde_json::from_str(DATA).unwrap()
	}

	const DATA: &str = r#"
        {
        	"announcement_at_start": "今期の__CONTENTS__は今日からです！",
        	"announcement_at_end": "今期の__CONTENTS__は今日までです！",
        	"contents": [
            	{
            		"id": "theatrical_chronicle",
            		"display": " :m_nasubimera: シアトリカルクロニクル",
            		"days": [1, 15]
            	},
            	{
            		"id": "mato",
            		"display": "不思議の魔塔",
            		"days": [1]
            	}
        	]
        }
	"#;
}
