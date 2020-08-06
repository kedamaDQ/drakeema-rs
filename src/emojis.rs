use std::str::FromStr;
use mastors::prelude::*;
use mastors::api::v1::custom_emojis;
use rand::Rng;
use crate::{
	Error,
	Result,
};

const CUSTOM_EMOJI_CATEGORIES_REGEX: &str = "^(?:モンスター|キャラクター)$";

pub struct Emojis<'a> {
	conn: &'a Connection,
	re: regex::Regex,
	rand: rand::rngs::ThreadRng,
	inner: Vec<String>,
	cache: Vec<String>,
}

impl<'a> Emojis<'a> {
	pub fn load(conn: &'a Connection) -> Result<Self> {
		let re = regex::Regex::from_str(CUSTOM_EMOJI_CATEGORIES_REGEX)?;
		custom_emojis::get(conn)
			.send()
			.map(|ce| {
				Emojis {
					conn,
					re: re.clone(),
					rand: rand::thread_rng(),
					inner: Self::build_emojis(&re, &ce),
					cache: Self::build_emojis(&re, &ce),
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
			self.inner.remove(self.rand.gen::<usize>() % self.inner.len())
		}
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

	fn build_emojis(re: &regex::Regex, ce: &mastors::entities::Emojis) -> Vec<String> {
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_supply_emoji_forever() {
		let conn = Connection::from_file(".env.test.st").unwrap();
		let mut emojis = Emojis::load(&conn).unwrap();
		let blank = String::new();
		for _i in 0 .. 500 {
			assert_ne!(emojis.get(), blank);
		}
	}

	#[test]
	fn test_category_filtering() {
		let conn = Connection::from_file(".env.test.st").unwrap();
		let emojis = Emojis::load(&conn).unwrap();
		let emojis_orig = mastors::api::v1::custom_emojis::get(&conn).send().unwrap();

		let monster_emojis = emojis_orig.iter()
			.filter(|e| e.category().is_some() && e.category().unwrap() == "モンスター")
			.collect::<Vec<&mastors::entities::Emoji>>();
		let character_emojis = emojis_orig.iter()
			.filter(|e| e.category().is_some() && e.category().unwrap() == "キャラクター")
			.collect::<Vec<&mastors::entities::Emoji>>();

		assert_eq!(monster_emojis.len() + character_emojis.len(), emojis.len());
	}
}
