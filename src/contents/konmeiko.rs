use std::fs::File;
use std::io::BufReader;
use chrono:: { DateTime, Duration, Local };
use serde::Deserialize;
use crate::{
	Error,
	Result,
	resistances::Resistances,
	utils::transform_string_to_regex,
};
use super::{
	Announcer,
	AnnouncementCriteria,
	Responder,
	ResponseCriteria,
};

const DATA: &str = "drakeema-data/contents/konmeiko.json";

#[derive(Debug, Clone)]
pub struct Konmeiko<'a> {
	monsters: KonmeikoMonsters<'a>>,
	inner: KonmeikoJson,
}

impl<'a> Konmeiko<'a> {

	pub fn load() -> Result<Self> {
		let inner: KonmeikoJson = serde_json::from_reader(
			BufReader::new(File::open(DATA)?)
		)
		.map_err(|e| Error::UnpersebleJson(DATA.to_owned(), e))?;


	}

	Ok(Konmeiko {
		monsters: KonmeikoMonsters::new(innter.monsters)?,
		inner,
	})
}

impl<'a> std::ops::Deref for Konmeiko<'a> {
	type Target = KonmeikoJson;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

#[derive(Debug, Clone, Deserialize)]
struct KonmeikoJson {
	reference_date: DateTime<Local>,
	announcement: String,
	announcement_at_start: String,
	information: String,
	out_of_term: String,
	#[serde(deserialize_with = "transform_string_to_regex")]
	nickname_regex: regex::Regex,
	monsters: Vec<MonsterJson>,
}

#[derive(Debug, Clone)]
struct KonmeikoMonsters<'a> {
	inner: Vec<KonmeikoMonster<'a>,
}

impl<'a> KonmeikoMonsters<'a> {
	fn new(k_monsters: impl AsRef<[MonsterJson]>) -> Result<Self> {
		let mut inner: Vec<KonmeikoMonster<'a>> = Vec::new();
		let monsters = crate::monsters();

		for monster in k_monsters.as_ref() {
			match monsters.get(&monster.monster_id) {
				Some(m) => inner.push(KonmeikoMonster {
					id: monster.id.to_owned(),
					monster: m,
				}),
				None => return Err(
					Error::UnknownMonsterId(DATA, monster.monster_id.clone())
				),
			}
		}

		Ok(KonmeikoMonsters {
			inner,
		})
	}
}

impl<'a> std::ops::Deref for KonmeikoMonsters<'a> {
	type Target = Vec<KonmeikoMonster<'a>>;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

#[derive(Debug, Clone)]
struct KonmeikoMonster<'a> {
	id: String,
	monster: &'a Monster,
}

#[derive(Debug, Clone, Deserialize)]
struct MonsterJson {
	id: String,
	monster_id: String,
}
