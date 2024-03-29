use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use feed_rs::model::Entry as FeedEntry;
use mastors::entities::Visibility;
use regex::Regex;
use reqwest::blocking::Client;
use serde::Deserialize;
use url::Url;
use crate::{
	Error,
	Message,
	Result,
	tmp_file,
	utils::transform_vec_string_to_vec_regex,
};

const DATA: &str = "drakeema-data/features/announcement/feeds.json";

#[derive(Debug, Clone)]
pub struct FeedsWorker{
	feeds: Arc<Feeds>,
	announcement_interval_secs: u64,
	post_interval_secs: u64,
}

impl FeedsWorker {
	pub fn load() -> Result<Self> {
		info!("Initializing FeedWorker");

		let json: FeedAnnouncementJson = serde_json::from_reader(
			BufReader::new(File::open(DATA)?)
		)
		.map_err(|e| Error::UnparseableJson(DATA.to_owned(), e))?;

		Ok(FeedsWorker{
			feeds: Arc::new(Feeds::new(json.feeds, json.title_regexes, json.user_agent)?),
			announcement_interval_secs: json.announcement_interval_secs,
			post_interval_secs: json.post_interval_secs,
		})
	}

	pub fn start(&self, tx: mpsc::Sender<Message>) {
		let post_interval_secs = self.post_interval_secs;
		let announcement_interval_secs = self.announcement_interval_secs;

		let feeds = Arc::clone(&self.feeds);
		thread::spawn(move || { loop {
			info!("Start announcing about feeds");
			match feeds.fetch_entries() {
				Ok(entries) => {
					for entry in entries {
						info!("Found entry to announce: {}", entry.title);

						tx.send(Message::Status{
							text: entry.build_text(),
							visibility: Visibility::Public,
							mention: None,
							in_reply_to_id: None,
							poll_options: None,
						}).unwrap();

						thread::sleep(Duration::from_secs(post_interval_secs));
					}
				},
				Err(e) => {
					error!("Failed to get feed: {}", e);
				}
			}

			info!("Next announcement about feeds will be in {} secs", announcement_interval_secs);
			thread::sleep(Duration::from_secs(announcement_interval_secs));
		}});
	}
}

#[derive(Debug, Clone)]
struct Feeds {
	client: Client,
	feeds: Vec<Feed>,
	regexes: Vec<Regex>,
}

impl Feeds {
	fn new(feeds: Vec<Feed>, regexes: Vec<Regex>, user_agent: Option<String>) -> Result<Self> {
		let client = if let Some(ua) = user_agent {
			Client::builder()
				.user_agent(ua)
				.build()?
		} else {
			Client::new()
		};

		Ok(Feeds {
			client,
			feeds,
			regexes,
		})
	}

	fn fetch_entries(&self) -> Result<Vec<Entry>> {
		let mut entries: Vec<Entry> = Vec::new();

		for feed_conf in self.feeds.iter() {
			let feed = feed_rs::parser::parse(
				self.client.get(feed_conf.url.clone()).send()?.text()?.as_bytes()
			)
			.map_err(|e| Error::UnparseableFeed(feed_conf.url.to_string(), e))?;

			debug!("Load last check ID from {}", &feed_conf.tmp);
			let last_id = tmp_file::load_tmp_as_string(&feed_conf.tmp)?;

			match self.search_new_items(&feed.entries, last_id) {
				Some(items) => {
					info!("Found {} new entries on {}", items.len(), feed_conf.title);

					debug!("Write last checked ID to {}", &feed_conf.tmp);
					tmp_file::save_tmp(&feed_conf.tmp, &items[0].id)?;
					items
				},
				None => {
					continue
				}
			}
			.iter()
				.filter(|e| e.title.is_some() && !e.links.is_empty())
				.filter(|e| {
					self.regexes.iter()
						.any(|r| r.is_match(&e.title.as_ref().unwrap().content))
				})
				.for_each(|e| entries.push(Entry::new(e)));
		}

		Ok(entries)
	}

	fn search_new_items<'b>(&self, feed: &'b [FeedEntry], id: Option<String>) -> Option<&'b [FeedEntry]> {
		let id = match id {
			Some(id) => id,
			None => return Some(feed),
		};

		if let Some((i, _)) = feed.iter()
			.enumerate()
			.find(|(_, item)| item.id == id) {
			
			if i == 0 {
				None
			} else {
				Some(&feed[.. i])
			}
		} else {
			Some(feed)
		}
	}
}

#[derive(Debug, Clone)]
struct Entry {
	title: String,
	authors: Option<String>,
	link: String,
}

impl Entry {
	fn new(entry: &FeedEntry) -> Self {
		let authors = if entry.authors.is_empty() {
			None
		} else {
			Some(entry.authors.iter()
				.map(|a| a.name.to_owned())
				.collect::<Vec<String>>()
				.join(", ")
			)
		};

		Entry {
			title: entry.title.as_ref().unwrap().content.to_owned(),
			authors,
			link: entry.links.get(0).unwrap().href.to_owned(),
		}
	}

	fn build_text(&self) -> String {
		if let Some(authors) = self.authors.as_ref() {
			format!("{} [{}]\n\n{}", self.title, authors, self.link)
		} else {
			format!("{}\n\n{}", self.title, self.link)
		}
	}
}

#[derive(Debug, Clone, Deserialize)]
struct FeedAnnouncementJson {
	feeds: Vec<Feed>,
	#[serde(deserialize_with = "transform_vec_string_to_vec_regex")]
	title_regexes: Vec<regex::Regex>,
	announcement_interval_secs: u64,
	post_interval_secs: u64,
	user_agent: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct Feed {
	title: String,
	url: Url,
	tmp: String,
}

#[cfg(test)]
mod tests {
}

