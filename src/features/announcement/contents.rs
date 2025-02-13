use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;
use std::time::Duration as StdDuration;
use chrono::{ DateTime, Duration, Local, NaiveTime };
use mastors::entities::Visibility;
use serde::Deserialize;
use crate::{
	Error,
	Message,
	Result,
	contents::*,
};

const DATA: &str = "drakeema-data/features/announcement/contents.json";

pub struct ContentsWorker {
	contents: Arc<Vec<Box<dyn Announcer>>>,
	announcement_times: Arc<AnnouncementTimes>,
}

impl ContentsWorker {
	pub fn load()-> Result<Self> {
		info!("Initialize ContentsWorker");

		let json: Json = serde_json::from_reader(
			BufReader::new(File::open(DATA)?)
		)
		.map_err(|e| Error::UnparseableJson(DATA.to_owned(), e))?;

		let contents: Vec<Box<dyn Announcer>> = vec![
			Box::new(PeriodicContents::load()?),
			Box::new(MonthlyContents::load()?),
			Box::new(Seishugosha::load()?),
			Box::new(Jashin::load()?),
			Box::new(Panigulm::load()?),
			Box::new(Konmeiko::load()?),
			Box::new(WeeklyActivity::load()?),
		];

		Ok(ContentsWorker {
			contents: Arc::new(contents),
			announcement_times: Arc::new(AnnouncementTimes::new(json.announcement_times)),
		})
	}

	pub fn start(&self, tx: mpsc::Sender<Message>) {
		let contents = Arc::clone(&self.contents);
		let announcement_times = Arc::clone(&self.announcement_times);

		thread::spawn(move || {loop {
			let duration_secs = announcement_times.duration_secs(Local::now());

			info!("Next announcement about contents will be in {} secs", duration_secs);
			thread::sleep(StdDuration::from_secs(duration_secs));

			let criteria = AnnouncementCriteria::new(Local::now());

			info!("Start announcing about contents: {:?}", criteria);
			let text = contents.iter()
				.map(|c| c.announce(&criteria))
				.flatten()
				.collect::<Vec<String>>()
				.join("\n\n");

			if !text.is_empty() {
				tx.send(Message::Status{
					text,
					visibility: Visibility::Public,
					mention: None,
					in_reply_to_id: None,
					poll_options: None,
				}).unwrap();
			}

			// Prevent runaway due to time error
			thread::sleep(StdDuration::from_secs(10));
		}});
	}
}

struct AnnouncementTimes {
	inner: Vec<NaiveTime>,
}

impl AnnouncementTimes {
	pub fn new(mut times: Vec<NaiveTime>) -> Self {
		times.sort();
		AnnouncementTimes {
			inner: times,
		}
	}

	pub fn duration_secs(&self, now: DateTime<Local>) -> u64 {
		let t = match self.inner.iter()
			.rev()
			.find(|t| t > &&now.time())
		{
			Some(time) => {
				now.date_naive().and_time(*time)
			},
			None => {
				(now.date_naive() + Duration::days(1)).and_time(*self.inner.get(0).unwrap())
			},
		};
		(t - now.date_naive().and_time(now.time())).num_seconds() as u64
	}
}

#[derive(Debug, Clone, Deserialize)]
struct Json {
	announcement_times: Vec<NaiveTime>,
}

#[cfg(test)]
mod tests {
	use super::*;
	use chrono::offset::TimeZone;

	#[test]
	fn test_duration_secs() {
		let at = at();
		let dt = Local.with_ymd_and_hms(2020, 9, 6, 6, 1, 30).unwrap();
		assert_eq!(at.duration_secs(dt), 43200);

		let dt = Local.with_ymd_and_hms(2020, 9, 6, 6, 1, 31).unwrap();
		assert_eq!(at.duration_secs(dt), 43199);

		let dt = Local.with_ymd_and_hms(2020, 9, 6, 6, 1, 29).unwrap();
		assert_eq!(at.duration_secs(dt), 43201);

		let dt = Local.timestamp_millis_opt(
			Local.with_ymd_and_hms(2020, 9, 6, 6, 1, 29).unwrap().timestamp_millis() + 1
		).unwrap();
		assert_eq!(at.duration_secs(dt), 43200);
	}

	fn at() -> AnnouncementTimes {
		AnnouncementTimes::new(
			serde_json::from_str::<Json>(DATA).unwrap().announcement_times
		)
	}

	fn _data() -> ContentsWorker {
		ContentsWorker {
			contents: Arc::new(Vec::new()),
			announcement_times: Arc::new(
				AnnouncementTimes::new(serde_json::from_str::<Json>(DATA).unwrap().announcement_times)
			)
		}
	}

	const DATA: &str = r#"
		{
			"announcement_times": [
				"06:01:30",
				"18:01:30"
			]
		}
	"#;
}
