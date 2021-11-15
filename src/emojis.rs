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

const DATA: &str = "drakeema-data/emojis.json";

#[derive(Debug, Clone)]
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
		info!("Initialize Emojis");
		let config: EmojiConfig = serde_json::from_reader(
			BufReader::new(File::open(DATA)?)
		)
		.map_err(|e| Error::UnparseableJson(DATA.to_owned(), e))?;

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
			.map_err(Error::MastorsApi)
	}

	pub fn emoji(&mut self) -> String {
		if self.inner.is_empty() {
			self.refresh();
		}

		if self.inner.is_empty() {
			String::new()
		} else {
			let shortcode = self.inner.remove(self.rand.gen::<usize>() % self.inner.len());
			trace!("Consumes an emoji: {}, remains: {}", shortcode, self.inner.len());
			String::from(":") + shortcode.as_str() + ":"
		}
	}

	pub fn emojify(&mut self, text: impl Into<String>) -> String {
		let mut text = text.into();
		trace!("Start emojify: {}", text);

		let placeholder = self.placeholder.clone();
		while text.contains(placeholder.as_str()) {
			text = text.replacen(placeholder.as_str(), self.emoji().as_str(), 1);
		}
		text
	}

	fn refresh(&mut self) {
		info!("Start refresh Emojis: inner.len: {}, cache.len: {}", self.len(), self.cache.len());

		match custom_emojis::get(self.conn).send() {
			Ok(ce) => {
				self.inner = Self::build_emojis(&self.re, &ce);
				self.cache = Self::build_emojis(&self.re, &ce);
				info!("Emojis refresh completed: remains: {}", self.inner.len())
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
pub struct EmojiConfig {
	placeholder: String,
	#[serde(deserialize_with = "transform_string_to_regex")]
	category_regex: regex::Regex,
}

#[cfg(test)]
pub(crate) mod tests {
	use super::*;

	#[test]
	#[ignore] // Need to server connection at refresh()
	fn test_supply_emoji_forever() {
		let conn_dummy = Connection::from_file(".env.test").unwrap();
		let mut emojis = data(&conn_dummy);
		let blank = String::new();

		for _i in 0 .. 500 {
			assert_ne!(emojis.emoji(), blank);
		}
	}

	#[test]
	fn test_category_filtering() {
		let conn_dummy = Connection::from_file(".env.test").unwrap();
		let emojis = data(&conn_dummy);
		let emojis_orig: mastors::entities::Emojis = serde_json::from_str(DATA).unwrap();

		let monster_emojis = emojis_orig.iter()
			.filter(|e| e.category().is_some() && e.category().unwrap() == "モンスター");
		let character_emojis = emojis_orig.iter()
			.filter(|e| e.category().is_some() && e.category().unwrap() == "キャラクター");

		assert_eq!(monster_emojis.count() + character_emojis.count(), emojis.len());
	}

	pub(crate) fn data(conn: &Connection) -> super::Emojis {
		let config: EmojiConfig = serde_json::from_str(CONFIG).unwrap();
		let ce: mastors::entities::Emojis = serde_json::from_str(DATA).unwrap();
		super::Emojis {
			conn,
			placeholder: config.placeholder.clone(),
			re: config.category_regex.clone(),
			rand: rand::thread_rng(),
			inner: super::Emojis::build_emojis(&config.category_regex, &ce),
			cache: super::Emojis::build_emojis(&config.category_regex, &ce),
		}
	}

	const CONFIG: &str = r#"{
		"placeholder": "__EMOJI__",
		"category_regex": "^(?:モンスター|キャラクター)$"
	}"#;

	const DATA: &str = r#"
        [
         {
            "shortcode": "d_palsy01",
            "url": "https://storest.foresdon.jp/custom_emojis/images/000/000/134/original/fdd_palsy.png",
            "static_url": "https://storest.foresdon.jp/custom_emojis/images/000/000/134/static/fdd_palsy.png",
            "visible_in_picker": true,
            "category": "ステータス"
          },
          {
            "shortcode": "d_illusion05",
            "url": "https://storest.foresdon.jp/custom_emojis/images/000/000/150/original/fdd_illusion.png",
            "static_url": "https://storest.foresdon.jp/custom_emojis/images/000/000/150/static/fdd_illusion.png",
            "visible_in_picker": true,
            "category": "ステータス"
          },
          {
            "shortcode": "m_bubblemetal",
            "url": "https://storest.foresdon.jp/custom_emojis/images/000/000/053/original/fdm_bubblemetal.png",
            "static_url": "https://storest.foresdon.jp/custom_emojis/images/000/000/053/static/fdm_bubblemetal.png",
            "visible_in_picker": true,
            "category": "モンスター"
          },
          {
            "shortcode": "c_porampan",
            "url": "https://storest.foresdon.jp/custom_emojis/images/000/000/207/original/ba9b2e733c7890f4.png",
            "static_url": "https://storest.foresdon.jp/custom_emojis/images/000/000/207/static/ba9b2e733c7890f4.png",
            "visible_in_picker": true,
            "category": "キャラクター"
          },
          {
            "shortcode": "s_tresurewhite01",
            "url": "https://storest.foresdon.jp/custom_emojis/images/000/000/113/original/fds_tresurewhite.png",
            "static_url": "https://storest.foresdon.jp/custom_emojis/images/000/000/113/static/fds_tresurewhite.png",
            "visible_in_picker": true,
            "category": "おたから"
          },
          {
            "shortcode": "t_fairy",
            "url": "https://storest.foresdon.jp/custom_emojis/images/000/000/214/original/1dbb828a0083626b.png",
            "static_url": "https://storest.foresdon.jp/custom_emojis/images/000/000/214/static/1dbb828a0083626b.png",
            "visible_in_picker": true,
            "category": "看板"
          },
          {
            "shortcode": "t_arms",
            "url": "https://storest.foresdon.jp/custom_emojis/images/000/000/215/original/7cce5ce61b645352.png",
            "static_url": "https://storest.foresdon.jp/custom_emojis/images/000/000/215/static/7cce5ce61b645352.png",
            "visible_in_picker": true,
            "category": "看板"
          },
          {
            "shortcode": "m_momontaru01",
            "url": "https://storest.foresdon.jp/custom_emojis/images/000/000/220/original/b44a9fc1c177f77b.png",
            "static_url": "https://storest.foresdon.jp/custom_emojis/images/000/000/220/static/b44a9fc1c177f77b.png",
            "visible_in_picker": true
          },
          {
            "shortcode": "m_hashiraorient",
            "url": "https://storest.foresdon.jp/custom_emojis/images/000/000/224/original/c94c556e64c765cd.png",
            "static_url": "https://storest.foresdon.jp/custom_emojis/images/000/000/224/static/c94c556e64c765cd.png",
            "visible_in_picker": true
          },
          {
            "shortcode": "b_church",
            "url": "https://storest.foresdon.jp/custom_emojis/images/000/000/028/original/fdb_church.5050.171022.png",
            "static_url": "https://storest.foresdon.jp/custom_emojis/images/000/000/028/static/fdb_church.5050.171022.png",
            "visible_in_picker": true,
            "category": "看板"
          },
          {
            "shortcode": "m_hoiminshibire",
            "url": "https://storest.foresdon.jp/custom_emojis/images/000/000/058/original/fdm_hoiminshibire.png",
            "static_url": "https://storest.foresdon.jp/custom_emojis/images/000/000/058/static/fdm_hoiminshibire.png",
            "visible_in_picker": true,
            "category": "モンスター"
          },
          {
            "shortcode": "i_sekaiju01",
            "url": "https://storest.foresdon.jp/custom_emojis/images/000/000/025/original/fdi_sekaiju.png",
            "static_url": "https://storest.foresdon.jp/custom_emojis/images/000/000/025/static/fdi_sekaiju.png",
            "visible_in_picker": true,
            "category": "アイテム"
          },
          {
            "shortcode": "m_drakee",
            "url": "https://storest.foresdon.jp/custom_emojis/images/000/000/003/original/fdm_drakee.png",
            "static_url": "https://storest.foresdon.jp/custom_emojis/images/000/000/003/static/fdm_drakee.png",
            "visible_in_picker": true,
            "category": "モンスター"
          },
          {
            "shortcode": "m_drakeema",
            "url": "https://storest.foresdon.jp/custom_emojis/images/000/000/004/original/fdm_drakeema.png",
            "static_url": "https://storest.foresdon.jp/custom_emojis/images/000/000/004/static/fdm_drakeema.png",
            "visible_in_picker": true,
            "category": "モンスター"
          }
        ]
	"#;
}
