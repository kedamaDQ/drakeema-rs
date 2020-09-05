use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;
use std::time::Duration as StdDuration;
use chrono::{ DateTime, Duration, Local, NaiveTime };
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
		.map_err(|e| Error::ParseJsonError(DATA.to_owned(), e))?;

		let contents: Vec<Box<dyn Announcer>> = vec![
			Box::new(PeriodicContents::load()?),
			Box::new(WeeklyContents::load()?),
			Box::new(MonthlyContents::load()?),
			Box::new(Seishugosha::load()?),
			Box::new(Jashin::load()?),
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
				.filter(|c| c.is_some())
				.map(|c| c.unwrap())
				.collect::<Vec<String>>()
				.join("\n\n");

			if !text.is_empty() {
				tx.send(Message::Status{ text, mention: None, in_reply_to_id: None}).unwrap();
			}
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
		match self.inner.iter()
			.rev()
			.find(|t| t > &&now.time())
		{
			Some(time) => {
				now.date().and_time(*time)
			},
			None => {
				(now.date() + Duration::days(1)).and_time(*self.inner.get(0).unwrap())
			},
		}
		.map(|t| (t - now).num_seconds() as u64)
		.unwrap()
	}
}

#[derive(Debug, Clone, Deserialize)]
struct Json {
	announcement_times: Vec<NaiveTime>,
}
