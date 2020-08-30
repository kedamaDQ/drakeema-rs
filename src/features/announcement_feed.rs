use std::fs::File;
use std::io::BufReader;
use feed_rs::model::{
	Entry,
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

const DATA: &str = "drakeema-data/announcement_feed.json";

#[derive(Debug, Clone)]
pub struct AnnouncementFeed {
	client: Client,
	feed_urls: Vec<FeedUrl>,
	title_regex: Vec<regex::Regex>,
}

impl AnnouncementFeed {
	pub fn load() -> Result<Self> {
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
			client,
			feed_urls,
			title_regex: json.title_regex,
		})
	}

	pub fn get(&self) -> Result<Vec<String>> {
		let mut entries = Vec::new();

		for feed_url in self.feed_urls.iter() {
			let feed = feed_rs::parser::parse(
				self.client.get(feed_url.url.clone()).send()?.text()?.as_bytes()
			)
			.map_err(|e| Error::ParseFeedError(feed_url.url.to_string(), e))?;
			println!("{:#?}", feed);

			let last_id = tmp_file::load_tmp_as_string(&feed_url.tmp)?;

			match self.search_new_items(&feed.entries, last_id) {
				Some(items) => {
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
						"{}\n{}",
						&e.title.as_ref().unwrap().content,
						&e.links.get(0).unwrap().href
					)
				));
		}

		println!("{:#?}", entries);
		Ok(entries)
	}

	fn search_new_items<'a>(&self, feed: &'a [Entry], id: Option<String>) -> Option<&'a [Entry]> {
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
	use super::*;

	#[test]
	#[ignore]
	fn test_test_test() {
		let a = AnnouncementFeed::load().unwrap();
		assert!(a.get().is_ok());
	}
}

#[derive(Debug, Clone, Deserialize)]
struct AnnouncementFeedJson {
	feed_urls: Vec<Url>,
	#[serde(deserialize_with = "transform_vec_string_to_vec_regex")]
	title_regex: Vec<regex::Regex>,
}

#[derive(Debug, Clone)]
struct FeedUrl {
	url: Url,
	tmp: String,
}
