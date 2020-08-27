use std::fs::File;
use std::io::BufReader;
use chrono::{ DateTime, Duration, Local };
use serde::Deserialize;
use crate::{
	Error,
	Result,
};
use crate::features::{ Announcer, AnnouncementCriteria };

const DATA: &str = "drakeema-data/contents/periodic_contents.json";

#[derive(Debug, Clone, Deserialize)]
pub struct PeriodicContents {
	announcement_at_day: String,
	announcement_at_day_before: String,
	contents: Vec<Content>,
}

impl PeriodicContents {
	pub fn load() -> Result<Self> {
		info!("Initialize PeriodicContents");

		serde_json::from_reader(
			BufReader::new(File::open(DATA)?)
		)
		.map_err(|e| Error::ParseJsonError(DATA.to_owned(), e))
	}

	fn contents_at_day(&self, at: &DateTime<Local>) -> String {
		self.contents(at, &self.announcement_at_day)
	}

	fn contents_at_day_before(&self, at: &DateTime<Local>) -> String {
		let tomorrow = *at + Duration::days(1);
		self.contents(&tomorrow, &self.announcement_at_day_before)
	}

	fn contents(&self, at: &DateTime<Local>, template: &str) -> String {
		use chrono::Datelike;

		let contents = self.contents.iter()
			.filter(|c| {
				c.months.as_ref().is_none() ||
				c.months.as_ref().unwrap().contains(&at.month())
			})
			.filter(|c| c.days.contains(&at.day()))
			.map(|c| c.display.to_owned())
			.collect::<Vec<String>>()
			.join("で");

		if contents.is_empty() {
			contents
		} else {
			template.replace("__CONTENTS__", &contents)
		}
	}
}

impl Announcer for PeriodicContents {
	fn announce(&self, criteria: &AnnouncementCriteria) -> Option<String> {
		trace!("Start to announce about periodic contents: {:?}", criteria);

		let contents = vec![
			self.contents_at_day(&criteria.at()),
			self.contents_at_day_before(&criteria.at()),
		].iter()
		.filter(|c| !c.is_empty())
		.map(|c| c.to_owned())
		.collect::<Vec<String>>()
		.join("\n");

		if contents.is_empty() {
			trace!("Nothing to announce about periodic_contents: {:?}", criteria);
			None
		} else {
			trace!("Found announcement about periodic contents: criteria: {:?}, announcement: {}", criteria, contents);
			Some(contents)
		}
	}
}

#[derive(Debug, Clone, Deserialize)]
struct Content {
	id: String,
	display: String,
	months: Option<Vec<u32>>,
	days: Vec<u32>,
}

#[cfg(test)]
mod tests {
	use super::*;
	use chrono::offset::TimeZone;

	#[test]
	fn test_contents_at_day_is_exist() {
		let pc = data();

		let an = pc.announce(
			&AnnouncementCriteria::new(Local.ymd(2020, 8, 10).and_hms(12, 0, 0))
		);
		assert_eq!(an.unwrap(), "テンの日です！");

		let an = pc.announce(
			&AnnouncementCriteria::new(Local.ymd(2020, 8, 12).and_hms(12, 0, 0))
		);
		assert_eq!(an.unwrap(), "じゅうににちで12日です！");
	}

	#[test]
	fn test_contents_at_day_before_is_exist() {
		let pc = data();

		let an = pc.announce(
			&AnnouncementCriteria::new(Local.ymd(2020, 8, 9).and_hms(12, 0, 0))
		);
		assert_eq!(an.unwrap(), "明日はテンの日です！");
	}

	#[test]
	fn test_contents_at_day_and_at_day_before_are_exist() {
		let pc = data();

		let an = pc.announce(
			&AnnouncementCriteria::new(Local.ymd(2020, 2, 9).and_hms(12, 0, 0))
		);
		assert_eq!(an.unwrap(), "プクの日です！\n明日はテンの日です！");
	}

	#[test]
	fn test_contents_is_nothing() {
		let pc = data();

		let an = pc.announce(
			&AnnouncementCriteria::new(Local.ymd(2020, 2, 5).and_hms(12, 0, 0))
		);
		assert!(an.is_none());
	}

	fn data() -> PeriodicContents {
		serde_json::from_str(DATA).unwrap()
	}

	const DATA: &str = r#"
        {
        	"announcement_at_day": "__CONTENTS__です！",
        	"announcement_at_day_before": "明日は__CONTENTS__です！",
        	"contents": [
            	{
            		"id": "tens_day",
            		"display": "テンの日",
            		"months": null,
            		"days": [10]
				},
            	{
            		"id": "tens_day",
            		"display": "じゅうににち",
            		"months": null,
            		"days": [12]
            	},
            	{
            		"id": "tens_day",
            		"display": "12日",
            		"months": null,
            		"days": [12]
            	},
            	{
            		"id": "pukus_day_monthly",
            		"display": "プクの日",
            		"months": null,
            		"days": [29]
            	},
            	{
            		"id": "pukus_day_yearly",
            		"display": "プクの日",
            		"months": [2],
            		"days": [9]
            	},
            	{
            		"id": "foresdon_anniversary",
            		"display": "フォレスドンの誕生日",
            		"months": [6],
            		"days": [27]
            	}
        	]
        }
	"#;
}
