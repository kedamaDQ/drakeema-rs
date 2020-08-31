use std::fs::File;
use std::io::BufReader;
use std::thread;
use std::time::Duration;
use feed_rs::model::{
	Entry,
};
use mastors::{
	Connection,
	api::v1::statuses,
};
use reqwest::blocking::{
	Client,
};
use serde::Deserialize;
use url::Url;
use crate::{
	Error,
	Result,
	tmp_file,
	utils::transform_vec_string_to_vec_regex,
};
use super::{
	AnnouncementCriteria,
	Announcer,
};

const DATA: &str = "drakeema-data/announcement_feed.json";

#[derive(Debug, Clone)]
pub struct AnnouncementFeed<'a> {
	conn: &'a Connection,
	client: Client,
	feed_urls: Vec<FeedUrl>,
	title_regex: Vec<regex::Regex>,
	post_interval_secs: u64,
}

impl<'a> AnnouncementFeed<'a> {
	pub fn load(conn: &'a Connection) -> Result<Self> {
		info!("Initializing AnnouncementFeed");

		let json: AnnouncementFeedJson = serde_json::from_reader(
			BufReader::new(File::open(DATA)?)
		)
		.map_err(|e| Error::ParseJsonError(DATA.to_owned(), e))?;

		let client = Client::new();

		let feed_urls = json.feed_urls.iter()
			.map(|u| FeedUrl {
				url: u.clone(),
				tmp: u.to_string().replace("/", "-").replace(":", "-").replace("*", "-") + ".tmp"
			})
			.collect::<Vec<FeedUrl>>();

		Ok(AnnouncementFeed {
			conn,
			client,
			feed_urls,
			title_regex: json.title_regex,
			post_interval_secs: json.post_interval_secs,
		})
	}

	pub fn announce(&self, criteria: &AnnouncementCriteria) -> Result<()> {
		info!("Start to announce feeds");
		for text in self.get()? {
			info!("Announce: {}", text.replace("\n", ""));
			statuses::post(self.conn, text).send()?;
			thread::sleep(Duration::from_secs(self.post_interval_secs));
		}
		Ok(())
	}

	fn get(&self) -> Result<Vec<String>> {
		trace!("Start to get new feeds");

		let mut entries = Vec::new();

		for feed_url in self.feed_urls.iter() {
			info!("Check {}", &feed_url.url);

			let feed = feed_rs::parser::parse(
				self.client.get(feed_url.url.clone()).send()?.text()?.as_bytes()
			)
			.map_err(|e| Error::ParseFeedError(feed_url.url.to_string(), e))?;

			trace!("Load last checked ID from {}", &feed_url.tmp);
			let last_id = tmp_file::load_tmp_as_string(&feed_url.tmp)?;
			trace!("Last checked ID: {:?}", last_id);

			match self.search_new_items(&feed.entries, last_id) {
				Some(items) => {
					info!("Found {} new entries", items.len());
					trace!("Write last checked ID to {}", &feed_url.tmp);

					tmp_file::save_tmp(&feed_url.tmp, &items[0].id)?;
					items
				},
				None => {
					continue
				}
			}.iter()
				.filter(|e| e.title.is_some() && !e.links.is_empty())
				.filter(|e| {
					self.title_regex.iter()
						.any(|r| r.is_match(&e.title.as_ref().unwrap().content))
				})
				.for_each(|e| entries.push(
					format!(
						"{}\n\n{}",
						&e.title.as_ref().unwrap().content,
						&e.links.get(0).unwrap().href
					)
				));
		}

		info!("Found {} new entries", entries.len());
		Ok(entries)
	}

	fn search_new_items<'b>(&self, feed: &'b [Entry], id: Option<String>) -> Option<&'b [Entry]> {
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
				Some(&feed[.. i - 1])
			}
		} else {
			Some(feed)
		}

	}
}

#[cfg(test)]
mod tests {
}

#[derive(Debug, Clone, Deserialize)]
struct AnnouncementFeedJson {
	feed_urls: Vec<Url>,
	#[serde(deserialize_with = "transform_vec_string_to_vec_regex")]
	title_regex: Vec<regex::Regex>,
	post_interval_secs: u64,
}

#[derive(Debug, Clone)]
struct FeedUrl {
	url: Url,
	tmp: String,
}
