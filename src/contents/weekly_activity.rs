use std::fs::File;
use std::io::BufReader;
use chrono::{ Duration, Local, NaiveDateTime };
use mastors::prelude::*;
use mastors::api::v1::instance::activity;
use serde::Deserialize;
use crate::{
	Error,
	Result,
	tmp_file,
};
use crate::features::{ Announcement, AnnouncementCriteria };

const DATA: &str = "drakeema-data/contents/weekly_activity.json";
const TMP: &str = "weekly_activity.tmp";

#[derive(Debug, Clone, Deserialize)]
pub struct WeeklyActivity {
	announcement: String,
}

impl WeeklyActivity {
	pub fn load() -> Result<Self> {
		info!("Initialize WeeklyActivity");

		serde_json::from_reader(
			BufReader::new(File::open(DATA)?)
		)
		.map_err(|e| Error::ParseJsonError(DATA.to_owned(), e))
	}
}

impl Announcement for WeeklyActivity {
	fn announcement(&self, criteria: &AnnouncementCriteria) -> Option<String> {
		trace!("Start to announce about weekly activity: {:?}", criteria);

		use chrono::offset::TimeZone;

		let conn = match Connection::from_file(crate::ENV_FILE) {
			Ok(conn) => conn,
			Err(e) => {
				error!("Failed to get the connection: {}", e);
				return None;
			},
		};

		let latest_activity = match activity::get(&conn).send() {
			Ok(activities) => {
				match activities.get(1) {
					Some(a) => *a,
					None => return None,
				}
			},
			Err(e) => {
				error!("Failed to get the activity: {}", e);
				return None;
			},
		};
		trace!("Latest activity: {:?}", latest_activity);

		let last_announced = match tmp_file::load_tmp_as_i64(TMP) {
			Ok(opt) => match opt {
				Some(data) => data,
				None => 0,
			},
			Err(e) => {
				error!("Failed to get last announced date: {}", e);
				return None;
			},
		};
		trace!("Last announced: {:?}", last_announced);

		if latest_activity.week() <= last_announced {
			trace!("Nothing to announce about weekly activity: {:?}", criteria);
			return None;
		}

		let start_date = Local.from_utc_datetime(
			&NaiveDateTime::from_timestamp(latest_activity.week(), 0)
		).date();
		let end_date = start_date + Duration::days(6);

		trace!("Week to announce: from: {}, to: {}", start_date, end_date);

		if let Err(e) = tmp_file::save_tmp(
			TMP, latest_activity.week().to_string()
		) {
			error!("Failed to write last announced date: {}", e);
			return None;
		}

		let announcement = self.announcement
			.replace("__START_DATE__", &start_date.format("%Y-%m-%d").to_string())
			.replace("__END_DATE__", &end_date.format("%Y-%m-%d").to_string())
			.replace("__ACTIVE_USER__", &latest_activity.logins().to_string())
			.replace("__STATUS_COUNT__", &latest_activity.statuses().to_string());
		
		trace!("Found announcement for weekly activity: criteria: {:?}, announcement: {}", criteria, announcement);

		Some(announcement)
	}
}

#[derive(Debug, Clone, Deserialize)]
struct WeeklyActivityJson {
	announcement: String,
}
