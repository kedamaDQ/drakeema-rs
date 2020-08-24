use std::fs::File;
use std::io::BufReader;
use chrono::{ DateTime, Duration, Local };
use serde::Deserialize;
use crate::{
	Error,
	Result,
};
use crate::features::{ Announcement, AnnouncementCriteria };

const DATA: &str = "drakeema-data/contents/weekly_contents.json";

#[derive(Debug, Clone, Deserialize)]
pub struct WeeklyContents {
	announcement_at_start: String,
	announcement_at_end: String,
	contents: Vec<WeeklyContent>,
}

use chrono::Datelike;

impl WeeklyContents {
	pub fn load() -> Result<Self> {
		serde_json::from_reader(
			BufReader::new(File::open(DATA)?)
		)
		.map_err(|e| Error::ParseJsonError(DATA.to_owned(), e))
	}

	fn contents_to_start(&self, at: DateTime<Local>) -> String {
		self.contents(&at.weekday().num_days_from_sunday(), &self.announcement_at_start)
	}

	fn contents_to_end(&self, at: DateTime<Local>) -> String {
		self.contents(&(at + Duration::days(1)).weekday().num_days_from_sunday(), &self.announcement_at_end)
	}

	fn contents(&self, wday_num: &u32, template: &str) -> String {
		let contents = self.iter()
			.filter(|wc| wc.reset_days.contains(wday_num))
			.map(|wc| String::from("__EMOJI__ ") + &wc.display)
			.collect::<Vec<String>>()
			.join("\n");

		if contents.is_empty() {
			contents
		} else {
			template.replace("__CONTENTS__", &contents)
		}
	}
}

impl Announcement for WeeklyContents {
	fn announcement(&self, criteria: &AnnouncementCriteria) -> Option<String> {
		let announcement = vec![
			self.contents_to_end(criteria.at()),
			self.contents_to_start(criteria.at())
		].iter()
		.filter(|c| !c.is_empty())
		.map(|c| c.to_owned())
		.collect::<Vec<String>>()
		.join("\n");

		if announcement.is_empty() {
			None
		} else {
			Some(announcement)
		}
	}
}

impl std::ops::Deref for WeeklyContents {
	type Target = Vec<WeeklyContent>;

	fn deref(&self) -> &Self::Target {
		&self.contents
	}
}

#[derive(Debug, Clone, Deserialize)]
pub struct WeeklyContent {
	id: String,
	display: String,
	reset_days: Vec<u32>,
}

#[cfg(test)]
mod tests {
	use super::*;
	use chrono::Local;
	use chrono::offset::TimeZone;

	#[test]
	fn test_contents_to_start_is_exist() {
		let wc = data();
		let sunday = Local.ymd(2020, 8, 23).and_hms(12, 0, 0);
		let monday = Local.ymd(2020, 8, 24).and_hms(12, 0, 0);
		let tuesday = Local.ymd(2020, 8, 25).and_hms(12, 0, 0);

		assert!(!(wc.contents_to_start(sunday).is_empty()));
		assert!(!(wc.contents_to_start(monday).is_empty()));
		assert!(!(wc.contents_to_start(tuesday).is_empty()));
	}

	#[test]
	fn test_contents_to_start_is_not_exist() {
		let wc = data();
		let wednesday = Local.ymd(2020, 8, 26).and_hms(12, 0, 0);
		let thursday = Local.ymd(2020, 8, 27).and_hms(12, 0, 0);
		let friday = Local.ymd(2020, 8, 28).and_hms(12, 0, 0);

		assert!(wc.contents_to_start(wednesday).is_empty());
		assert!(wc.contents_to_start(thursday).is_empty());
		assert!(wc.contents_to_start(friday).is_empty());
	}

	#[test]
	fn test_contents_to_end_is_exist() {
		let wc = data();
		let saturday = Local.ymd(2020, 8, 22).and_hms(12, 0, 0);
		let sunday = Local.ymd(2020, 8, 23).and_hms(12, 0, 0);
		let monday = Local.ymd(2020, 8, 24).and_hms(12, 0, 0);

		assert!(!(wc.contents_to_end(saturday).is_empty()));
		assert!(!(wc.contents_to_end(sunday).is_empty()));
		assert!(!(wc.contents_to_end(monday).is_empty()));
	}

	#[test]
	fn test_contents_to_end_is_not_exist() {
		let wc = data();
		let tuesday = Local.ymd(2020, 8, 25).and_hms(12, 0, 0);
		let wednesday = Local.ymd(2020, 8, 26).and_hms(12, 0, 0);
		let thursday = Local.ymd(2020, 8, 27).and_hms(12, 0, 0);

		assert!(wc.contents_to_end(tuesday).is_empty());
		assert!(wc.contents_to_end(wednesday).is_empty());
		assert!(wc.contents_to_end(thursday).is_empty());
	}

	fn data() -> WeeklyContents {
		serde_json::from_str(DATA).unwrap()
	}

	const DATA: &str = r#"
        {
			"announcement_at_start": "今週の……\n__CONTENTS__\n……は、今日からです！",
			"announcement_at_end": "今週の……\n__CONTENTS__\n……は、今日までです！",
			"contents": [
            	{
            		"id": "banma",
            		"display": "万魔の塔",
            		"reset_days": [0]
            	},
            	{
            		"id": "shutobatsu",
            		"display": "レンダーシア討伐隊",
            		"reset_days": [0]
            	},
            	{
            		"id": "pyramid",
            		"display": "ピラミッドの秘宝",
            		"reset_days": [1]
            	},
            	{
            		"id": "shiren",
            		"display": "試練の門",
            		"reset_days": [1]
            	},
            	{
            		"id": "ohke",
            		"display": "王家の迷宮",
            		"reset_days": [2]
            	},
            	{
            		"id": "tatsujin",
            		"display": "達人クエスト",
            		"reset_days": [0]
            	},
            	{
            		"id": "tarot",
            		"display": "モンスタータロット販売",
            		"reset_days": [0]
            	}
			]
		}
	"#;
}
