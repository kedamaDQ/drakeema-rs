use std::fs::File;
use std::io::BufReader;
use chrono::{ DateTime, Local, };
use serde::Deserialize;
use crate::{
	Error,
	Result,
	monsters::Monsters,
	utils::transform_string_to_regex,
};

const DATA: &str = "data/contents/seishugosha.json";

pub struct Seishugosha<'a, SeishugoshaJson> {
	monsters: &'a Monsters,
	inner: SeishugoshaJson,
}

impl<'a> Seishugosha<'a, SeishugoshaJson> {
	pub fn load(monsters: &'a Monsters) -> Result<Self> {
		let inner = match serde_json::from_reader(
			BufReader::new(File::open(DATA)?)
		) {
			Ok(sj) => sj,
			Err(e) => return Err(
				Error::ParseJsonError(DATA.to_owned(), e)
			),
		};

		Ok(Seishugosha {
			monsters,
			inner,
		})
	}

	pub fn announcement(&self, at: DateTime<Local>) -> String {
		let parts = self.seishugosha_monsters.iter()
			.map(|m| {
				let display = self.monsters.get(&m.monster_id)
					.expect("Unknown monster ID")
					.display();
				self.announcement.parts.clone()
					.replace("__NAME__", &display)
					.replace("__LEVEL__", self.level_name(at, m.offset))
			})
			.collect::<Vec<String>>()
			.join("\n");

		self.announcement.start.clone() + &parts + &self.announcement.end
	}

	pub fn information(&self, text: &str, at: DateTime<Local>) -> Option<String> {
		if self.is_match(text) {
			return Some(self.announcement(Local::now()));
		}

		let informations = self.seishugosha_monsters.iter()
			.map(|m| (
				self.monsters
					.get(&m.monster_id)
					.expect("Unknown monster ID"),
				m.offset
			))
			.filter(|(m, _o)| m.is_match(text))
			.map(|(m, o)| {
				self.information.clone()
					.replace("__NAME__", m.official_name())
					.replace("__LEVEL__", self.level_name(at, o))
					.replace("__RESISTANCES__", m.resistances().display(None::<Vec<String>>).as_str())
			})
			.collect::<Vec<String>>();
		
		if informations.is_empty() {
			None
		} else {
			Some(informations.join("\n"))
		}
	}

	fn is_match(&self, text: impl AsRef<str>) -> bool {
		self.nickname_regex.is_match(text.as_ref())
	}

	fn level_name(&self, at: DateTime<Local>, offset: i64) -> &str {
		let mut level_index = (self.elapsed_days(at) + offset) % self.level_names.len() as i64;
		if level_index < 0 {
			level_index += self.level_names.len() as i64;
		}
		self.level_names[level_index as usize].as_str()
	}

	fn elapsed_days(&self, at: DateTime<Local>) -> i64 {
		(at - self.reference_date).num_days()
	}
}

impl<'a, T> std::ops::Deref for Seishugosha<'a, T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

#[derive(Debug, Clone, Deserialize)]
pub struct SeishugoshaJson {
	reference_date: DateTime<Local>,
	level_names: Vec<String>,
	announcement: Announcement,
	information: String,
	#[serde(deserialize_with = "transform_string_to_regex")]
	nickname_regex: regex::Regex,
	seishugosha_monsters: Vec<SeishugoshaMonster>,
}

#[derive(Debug, Clone, Deserialize)]
struct Announcement {
	start: String,
	parts: String,
	end: String,
}

#[derive(Debug, Clone, Deserialize)]
struct SeishugoshaMonster {
	id: String,
	monster_id: String,
	offset: i64,
}
