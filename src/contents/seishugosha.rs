use std::fs::File;
use std::io::BufReader;
use chrono::{ DateTime, Local, };
use serde::Deserialize;
use crate::{
	Error,
	Result,
	monsters::{ Monster, Monsters },
	utils::transform_string_to_regex,
};

const DATA: &str = "data/contents/seishugosha.json";

pub struct Seishugosha<'a> {
	monsters: Vec<SeishugoshaMonster<'a>>,
	inner: SeishugoshaJson,
}

impl<'a> Seishugosha<'a> {
	pub fn load(monsters: &'a Monsters) -> Result<Self> {
		let inner: SeishugoshaJson = match serde_json::from_reader(
			BufReader::new(File::open(DATA)?)
		) {
			Ok(sj) => sj,
			Err(e) => return Err(
				Error::ParseJsonError(DATA.to_owned(), e)
			),
		};

		let mut monsters_vec: Vec<SeishugoshaMonster> = Vec::new();
		for monster in inner.seishugosha_monsters.iter() {
			match monsters.get(&monster.monster_id) {
				Some(m) => monsters_vec.push(SeishugoshaMonster {
					monster: &m,
					id: monster.id.to_owned(),
					offset: monster.offset,
				}),
				None => return Err(
					Error::UnknownMonsterIdError(DATA.to_owned(), monster.monster_id.clone())
				),
			}
		}

		Ok(Seishugosha {
			monsters: monsters_vec,
			inner,
		})
	}

	pub fn announcement(&self, at: DateTime<Local>) -> String {
		let parts = self.monsters.iter()
			.map(|m| {
				self.announcement.parts.clone()
					.replace("__NAME__", m.monster.display())
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

		let informations = self.monsters.iter()
			.filter(|m| m.is_match(text))
			.map(|m| {
				self.information.clone()
					.replace("__NAME__", m.official_name())
					.replace("__LEVEL__", self.level_name(at, m.offset))
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
		use chrono::Duration;

		if at < self.reference_date {
			(at - self.reference_date + Duration::nanoseconds(1)).num_days() - 1
		} else {
			(at - self.reference_date).num_days()
		}
	}
}

impl<'a> std::ops::Deref for Seishugosha<'a> {
	type Target = SeishugoshaJson;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

struct SeishugoshaMonster<'a> {
	id: String,
	monster: &'a Monster,
	offset: i64,
}

impl<'a> std::ops::Deref for SeishugoshaMonster<'a> {
	type Target = &'a Monster;

	fn deref(&self) -> &Self::Target {
		&self.monster
	}
}
#[derive(Debug, Clone, Deserialize)]
pub struct SeishugoshaJson {
	reference_date: DateTime<Local>,
	level_names: Vec<String>,
	announcement: AnnouncementJson,
	information: String,
	#[serde(deserialize_with = "transform_string_to_regex")]
	nickname_regex: regex::Regex,
	seishugosha_monsters: Vec<SeishugoshaMonsterJson>,
}

#[derive(Debug, Clone, Deserialize)]
struct AnnouncementJson {
	start: String,
	parts: String,
	end: String,
}

#[derive(Debug, Clone, Deserialize)]
struct SeishugoshaMonsterJson {
	id: String,
	monster_id: String,
	offset: i64,
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::monsters;
	use chrono::offset::TimeZone;

	#[test]
	fn test_positive() {
		let monsters = monsters::load().unwrap();
		let ssgs = load(&monsters);

		assert_eq!(
			ssgs.level_name(chrono::Local.ymd(2018, 4, 20).and_hms(6, 0, 0), 0),
			"Ⅰ"
		);

		// Edge of first day
		assert_eq!(
			ssgs.level_name(chrono::Local.ymd(2018, 4, 21).and_hms(5, 59, 59), 0),
			"Ⅰ"
		);

		assert_eq!(
			ssgs.level_name(chrono::Local.ymd(2018, 4, 21).and_hms(6, 0, 0), 0),
			"Ⅱ"
		);

		assert_eq!(
			ssgs.level_name(chrono::Local.ymd(2018, 4, 22).and_hms(6, 0, 0), 0),
			"Ⅲ"
		);

		assert_eq!(
			ssgs.level_name(chrono::Local.ymd(2018, 4, 23).and_hms(6, 0, 0), 0),
			"Ⅰ"
		);
	}

	#[test]
	fn test_negative() {
		let monsters = monsters::load().unwrap();
		let ssgs = load(&monsters);

		assert_eq!(
			ssgs.level_name(chrono::Local.ymd(2018, 4, 20).and_hms(6, 0, 0), 0),
			"Ⅰ"
		);

		// Edge of 1 day ago
		assert_eq!(
			ssgs.level_name(chrono::Local.ymd(2018, 4, 20).and_hms(5, 59, 59), 0),
			"Ⅲ"
		);

		assert_eq!(
			ssgs.level_name(chrono::Local.ymd(2018, 4, 19).and_hms(6, 0, 0), 0),
			"Ⅲ"
		);

		assert_eq!(
			ssgs.level_name(chrono::Local.ymd(2018, 4, 18).and_hms(6, 0, 0), 0),
			"Ⅱ"
		);

		assert_eq!(
			ssgs.level_name(chrono::Local.ymd(2018, 4, 17).and_hms(6, 0, 0), 0),
			"Ⅰ"
		);

		assert_eq!(
			ssgs.level_name(chrono::Local.ymd(2018, 4, 16).and_hms(6, 0, 0), 0),
			"Ⅲ"
		);

	}

	fn load(monsters: &Monsters) -> Seishugosha {
		let inner: SeishugoshaJson = serde_json::from_str(TEST_DATA).unwrap();
		let mut monsters_vec: Vec<SeishugoshaMonster> = Vec::new();
		for monster in inner.seishugosha_monsters.iter() {
			monsters_vec.push(SeishugoshaMonster {
				id: monster.id.clone(),
				offset: monster.offset,
				monster: monsters.get(&monster.monster_id).unwrap(),
			});
		}

		Seishugosha {
			monsters: monsters_vec,
			inner,
		}
	}


	const TEST_DATA: &str = r#"
        {
            "reference_date": "2018-04-20T06:00:00.000+09:00",
            "level_names": ["Ⅰ", "Ⅱ", "Ⅲ"],
            "announcement": {
                "start": "本日の 聖守護者の闘戦記 は……\n",
                "parts": "__NAME__：__LEVEL__",
                "end": "\n……です！"
            },
            "information": "本日の __NAME__ は __LEVEL__ です！あると良い耐性は __RESISTANCES__ です！",
            "nickname_regex": "(?:聖?守護者|(?:せい)?しゅごしゃ|(?:セイ)?シュゴシャ|(?:ｾｲ)?ｼｭｺﾞｼｬ|闘戦記|とうせんき|トウセンキ|ﾄｳｾﾝｷ)",
            "seishugosha_monsters": [
                {
                    "id": "regrog",
                    "monster_id": "seishugosha_regrog",
                    "offset": 0
                },
                {
                    "id": "scorpide",
                    "monster_id": "seishugosha_scorpide",
                    "offset": 2
                },
                {
                    "id": "jelzarg",
                    "monster_id": "seishugosha_jelzarg",
                    "offset": 1
                },
                {
                    "id": "gardodon",
                    "monster_id": "seishugosha_gardodon",
                    "offset": 1
                }
            ]
        }
	"#;
}
