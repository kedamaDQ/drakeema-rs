use std::fs::File;
use std::io::BufReader;
use mastors::prelude::*;
use mastors::api::v1::custom_emojis;
use rand::Rng;
use serde::Deserialize;
use crate::{
	Error,
	Result,
	utils::transform_string_to_regex,
};

const DATA: &str = "data/emojis.json";

pub struct Emojis<'a> {
	conn: &'a Connection,
	placeholder: String,
	re: regex::Regex,
	rand: rand::rngs::ThreadRng,
	inner: Vec<String>,
	cache: Vec<String>,
}

impl<'a> Emojis<'a> {
	pub fn load(conn: &'a Connection) -> Result<Self> {
		let config: EmojiConfig = match serde_json::from_reader(
			BufReader::new(File::open(DATA)?)
		) {
			Ok(ec) => ec,
			Err(e) => return Err(
				Error::ParseJsonError(DATA.to_owned(), e)
			)
		};

		custom_emojis::get(conn)
			.send()
			.map(|ce| {
				Emojis {
					conn,
					placeholder: config.placeholder.clone(),
					re: config.category_regex.clone(),
					rand: rand::thread_rng(),
					inner: Self::build_emojis(&config.category_regex, &ce),
					cache: Self::build_emojis(&config.category_regex, &ce),
				}
			})
			.map_err(Error::MastorsApiError)
	}

	pub fn get(&mut self) -> String {
		if self.inner.is_empty() {
			self.refresh();
		}

		if self.inner.is_empty() {
			String::new()
		} else {
			let shortcode = self.inner.remove(self.rand.gen::<usize>() % self.inner.len());
			String::from(":") + shortcode.as_str() + ":"
		}
	}

	pub fn emojify(&mut self, text: impl Into<String>) -> String {
		let mut text = text.into();
		let placeholder = self.placeholder.clone();
		while text.find(placeholder.as_str()).is_some() {
			text = text.replacen(placeholder.as_str(), self.get().as_str(), 1);
			println!("{}", text);
		}
		text
	}

	fn refresh(&mut self) {
		match custom_emojis::get(self.conn).send() {
			Ok(ce) => {
				self.inner = Self::build_emojis(&self.re, &ce);
				self.cache = Self::build_emojis(&self.re, &ce);
			},
			Err(e) => {
				error!("Can't get latest custom emojis: {}", e);
				self.inner = self.cache.clone();
			}
		}
	}

	fn build_emojis(re: &regex::Regex, ce: &[mastors::entities::Emoji]) -> Vec<String> {
		let emojis = ce.iter()
			.filter(|ce| ce.category().is_some() && re.is_match(ce.category().unwrap()))
			.map(|ce| ce.shortcode().to_owned())
			.collect::<Vec<String>>();
		
		if emojis.is_empty() {
			panic!("No emojis");
		}

		emojis
	}
}

impl<'a> std::ops::Deref for Emojis<'a> {
	type Target = Vec<String>;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

#[derive(Debug, Clone, Deserialize)]
struct EmojiConfig {
	placeholder: String,
	#[serde(deserialize_with = "transform_string_to_regex")]
	category_regex: regex::Regex,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_supply_emoji_forever() {
		let conn = Connection::from_file(".env.test.st").unwrap();
		let mut emojis = data(&conn);
		let blank = String::new();
		for _i in 0 .. 500 {
			assert_ne!(emojis.get(), blank);
		}
	}

	#[test]
	fn test_category_filtering() {
		let conn = Connection::from_file(".env.test.st").unwrap();
		let emojis = data(&conn);
		let emojis_orig = mastors::api::v1::custom_emojis::get(&conn).send().unwrap();

		let monster_emojis = emojis_orig.iter()
			.filter(|e| e.category().is_some() && e.category().unwrap() == "モンスター")
			.collect::<Vec<&mastors::entities::Emoji>>();
		let character_emojis = emojis_orig.iter()
			.filter(|e| e.category().is_some() && e.category().unwrap() == "キャラクター")
			.collect::<Vec<&mastors::entities::Emoji>>();

		assert_eq!(monster_emojis.len() + character_emojis.len(), emojis.len());
	}

	fn data(conn: &Connection) -> Emojis {
		let config: EmojiConfig = serde_json::from_str(CONFIG).unwrap();
		custom_emojis::get(conn)
			.send()
			.map(|ce| {
				Emojis {
					conn,
					placeholder: config.placeholder.clone(),
					re: config.category_regex.clone(),
					rand: rand::thread_rng(),
					inner: Emojis::build_emojis(&config.category_regex, &ce),
					cache: Emojis::build_emojis(&config.category_regex, &ce),
				}
			})
			.unwrap()

	}

	const CONFIG: &str = r#"{
		"placeholder": "__EMOJI__",
		"category_regex": "^(?:モンスター|キャラクター)$"
	}"#;
}
