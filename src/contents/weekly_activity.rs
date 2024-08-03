use std::fs::File;
use std::io::BufReader;
use chrono::{ Duration, Local, DateTime };
use mastors::prelude::*;
use mastors::api::v1::instance::activity;
use serde::Deserialize;
use crate::{
	Error,
	Result,
	tmp_file,
};
use super::{ Announcer, AnnouncementCriteria };

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
		.map_err(|e| Error::UnparseableJson(DATA.to_owned(), e))
	}
}

impl Announcer for WeeklyActivity {
	fn announce(&self, criteria: &AnnouncementCriteria) -> Option<String> {
		debug!("Start building announcement about WeeklyActivities: {:?}", criteria);

		use chrono::offset::TimeZone;

		let conn = match Connection::new() {
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
		debug!("Latest activity: {:?}", latest_activity);

		let last_announced = match tmp_file::load_tmp_as_i64(TMP) {
			Ok(opt) => opt.unwrap_or(0),
			Err(e) => {
				error!("Failed to get last announced date: {}", e);
				return None;
			},
		};
		debug!("Last announced: {:?}", last_announced);

		if latest_activity.week() <= last_announced {
			debug!("Nothing announcement about WeeklyActivity: {:?}", criteria);
			return None;
		}

		let start_date = Local.from_utc_datetime(
			&DateTime::from_timestamp(latest_activity.week(), 0).unwrap().naive_utc()
		).date_naive();
		let end_date = start_date + Duration::days(6);

		debug!("Week to announce: from: {}, to: {}", start_date, end_date);

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
		
		Some(announcement)
	}
}
