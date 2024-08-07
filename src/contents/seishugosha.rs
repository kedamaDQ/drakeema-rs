use std::fs::File;
use std::io::BufReader;
use chrono::{ DateTime, Local, };
use serde::Deserialize;
use crate::{
	Error,
	Result,
	monsters::Monster,
	utils::transform_string_to_regex,
};
use super::{
	Announcer,
	AnnouncementCriteria,
	Responder,
	ResponseCriteria,
};

const DATA: &str = "drakeema-data/contents/seishugosha.json";

#[derive(Debug, Clone)]
pub struct Seishugosha<'a> {
	monsters: SeishugoshaMonsters<'a>,
	inner: SeishugoshaJson,
}

impl<'a> Seishugosha<'a> {
	pub fn load() -> Result<Self> {
		info!("Initialize Seishugosha");

		let inner: SeishugoshaJson = serde_json::from_reader(
			BufReader::new(File::open(DATA)?)
		)
		.map_err(|e| Error::UnparseableJson(DATA.to_owned(), e))?;

		Ok(Seishugosha {
			monsters: SeishugoshaMonsters::new(&inner.monsters)?,
			inner,
		})
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

impl<'a> Announcer for Seishugosha<'a> {
	fn announce(&self, criteria: &AnnouncementCriteria) -> Option<String> {
		debug!("Start building announcement about Seishugosha: {:?}", criteria);

		let parts = self.monsters.iter()
			.map(|m| {
				self.announcement.parts.clone()
					.replace("__NAME__", m.monster.display())
					.replace("__LEVEL__", self.level_name(criteria.at(), m.offset))
			})
			.collect::<Vec<String>>()
			.join("\n");

		let announcement = 
			self.announcement.start.clone() +
			&parts +
			&self.announcement.end;
		
		Some(announcement)
	}
}

impl<'a> Responder for Seishugosha<'a> {
	fn respond(&self, criteria: &ResponseCriteria) -> Option<String> {
		debug!("Start to reaction about seishugosha: {:?}", criteria);

		if self.is_match(criteria.text()) {
			let reaction = self.announce(&AnnouncementCriteria::new(criteria.at()));
			info!("Text matched keywords of Seishugosha: {}", criteria.text());
			reaction
		} else {
			debug!("Nothing response about seishugosha: {:?}", criteria);
			None
		}
	}
}

impl<'a> std::ops::Deref for Seishugosha<'a> {
	type Target = SeishugoshaJson;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

#[derive(Debug, Clone)]
struct SeishugoshaMonsters<'a> {
	inner: Vec<SeishugoshaMonster<'a>>,
}

impl<'a> SeishugoshaMonsters<'a> {
	fn new(s_monsters: impl AsRef<[MonsterJson]>) -> Result<Self> {
		let mut inner: Vec<SeishugoshaMonster<'a>> = Vec::new();
		let monsters = crate::monsters();

		for monster in s_monsters.as_ref() {
			match monsters.get(&monster.monster_id) {
				Some(m) => inner.push(SeishugoshaMonster {
					id: monster.id.to_owned(),
					monster: m,
					offset: monster.offset,
				}),
				None => return Err(
					Error::UnknownMonsterId(DATA, monster.monster_id.clone())
				),
			}
		}

		Ok(SeishugoshaMonsters {
			inner,
		})
	}
}

impl<'a> std::ops::Deref for SeishugoshaMonsters<'a> {
	type Target = Vec<SeishugoshaMonster<'a>>;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

#[derive(Debug, Clone)]
struct SeishugoshaMonster<'a> {
	#[allow(unused)]
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
	announcement: AnnouncerJson,
	#[allow(dead_code)]
	information: String,
	#[serde(deserialize_with = "transform_string_to_regex")]
	nickname_regex: regex::Regex,
	monsters: Vec<MonsterJson>,
}

#[derive(Debug, Clone, Deserialize)]
struct AnnouncerJson {
	start: String,
	parts: String,
	end: String,
}

#[derive(Debug, Clone, Deserialize)]
struct MonsterJson {
	id: String,
	monster_id: String,
	offset: i64,
}

#[cfg(test)]
pub(crate) mod tests {
	use super::*;
	use chrono::offset::TimeZone;

	#[test]
	fn test_positive() {
		let ssgs = data();

		assert_eq!(
			ssgs.level_name(chrono::Local.with_ymd_and_hms(2018, 4, 20, 6, 0, 0).unwrap(), 0),
			"Ⅰ"
		);

		// Edge of first day
		assert_eq!(
			ssgs.level_name(chrono::Local.with_ymd_and_hms(2018, 4, 21, 5, 59, 59).unwrap(), 0),
			"Ⅰ"
		);

		assert_eq!(
			ssgs.level_name(chrono::Local.with_ymd_and_hms(2018, 4, 21, 6, 0, 0).unwrap(), 0),
			"Ⅱ"
		);

		assert_eq!(
			ssgs.level_name(chrono::Local.with_ymd_and_hms(2018, 4, 22, 6, 0, 0).unwrap(), 0),
			"Ⅲ"
		);

		assert_eq!(
			ssgs.level_name(chrono::Local.with_ymd_and_hms(2018, 4, 23, 6, 0, 0).unwrap(), 0),
			"Ⅰ"
		);
	}

	#[test]
	fn test_negative() {
		let ssgs = data();

		assert_eq!(
			ssgs.level_name(chrono::Local.with_ymd_and_hms(2018, 4, 20, 6, 0, 0).unwrap(), 0),
			"Ⅰ"
		);

		// Edge of 1 day ago
		assert_eq!(
			ssgs.level_name(chrono::Local.with_ymd_and_hms(2018, 4, 20, 5, 59, 59).unwrap(), 0),
			"Ⅲ"
		);

		assert_eq!(
			ssgs.level_name(chrono::Local.with_ymd_and_hms(2018, 4, 19, 6, 0, 0).unwrap(), 0),
			"Ⅲ"
		);

		assert_eq!(
			ssgs.level_name(chrono::Local.with_ymd_and_hms(2018, 4, 18, 6, 0, 0).unwrap(), 0),
			"Ⅱ"
		);

		assert_eq!(
			ssgs.level_name(chrono::Local.with_ymd_and_hms(2018, 4, 17, 6, 0, 0).unwrap(), 0),
			"Ⅰ"
		);

		assert_eq!(
			ssgs.level_name(chrono::Local.with_ymd_and_hms(2018, 4, 16, 6, 0, 0).unwrap(), 0),
			"Ⅲ"
		);

	}

	pub(crate) fn data<'a>() -> Seishugosha<'a> {
		let inner: SeishugoshaJson = serde_json::from_str(TEST_DATA).unwrap();
		Seishugosha {
			monsters: SeishugoshaMonsters::new(&inner.monsters).unwrap(),
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
            "monsters": [
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
