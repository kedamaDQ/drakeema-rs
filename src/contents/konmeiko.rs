use std::fs::File;
use std::io::BufReader;
use chrono:: { Datelike, DateTime, Duration, Local, LocalResult, TimeZone };
use serde::Deserialize;
use crate::{
	Error,
	monsters::Monster,
	Result,
	utils::transform_string_to_regex,
};
use super::{
	Announcer,
	AnnouncementCriteria,
	Responder,
	ResponseCriteria,
};

const DATA: &str = "drakeema-data/contents/konmeiko.json";
const START_TIME: u32 = 6;

#[derive(Debug, Clone)]
pub struct Konmeiko<'a> {
	monsters: KonmeikoMonsters<'a>,
	inner: KonmeikoJson,
}

impl<'a> Konmeiko<'a> {

	pub fn load() -> Result<Self> {
		info!("Initialize Konmeiko");

		let inner: KonmeikoJson = serde_json::from_reader(
			BufReader::new(File::open(DATA)?)
		)
		.map_err(|e| Error::UnparseableJson(DATA.to_owned(), e))?;

		Ok(Konmeiko {
			monsters: KonmeikoMonsters::new(&inner.monsters)?,
			inner,
		})
	}

	fn is_open(&self, at: DateTime<Local>) -> IsOpen {
		let term_in_hours: Duration = Duration::hours(self.term_in_hours);

		for d in self.days.iter() {
			if &at.day() == d {
				return IsOpen::FirstDay;
			}

			let term_start: DateTime<Local> = match at.timezone().with_ymd_and_hms(
				at.year(),
				at.month(),
				d.to_owned(),
				START_TIME,
				0,
				0,
			) {
				LocalResult::Single(ts) => ts,
				_ => {
					error!("DateTime returned not Single value");
					return IsOpen::OutOfTerm;
				},

			};

			let term_end: DateTime<Local> = match term_start.checked_add_signed(term_in_hours) {
				Some(te) => te,
				None => {
					error!("DateTime addition failed");
					return IsOpen::OutOfTerm;
				},
			};

			if term_start < at && at < term_end {
				return IsOpen::IsInTerm;
			}
		}

		IsOpen::OutOfTerm
	}

	fn is_match(&self, text: impl AsRef<str>) -> bool {
		self.nickname_regex.is_match(text.as_ref())
	}

	fn current_monster(&self, at: DateTime<Local>) -> &KonmeikoMonster {
		let index = if self.reference_date.year() == at.year() {
			(
				// number of past month's switching
				(at.month() as usize - self.reference_date.month() as usize) * 2 +
				// number of this month's switching
				at.day() as usize / 15
			 ) % self.monsters.len()
		} else {
			(
				// number of start year's switching
				(12 - self.reference_date.month() as usize) * 2 +
				// number of past year's switching
				(at.year() as usize - self.reference_date.year() as usize- 1) * 2 +
				// number of this year's switching
				(at.month() as usize - 1) * 2 +
				// number of this month's switching
				at.day() as usize / 15
			) % self.monsters.len()
		};

		self.monsters.get(index).unwrap()
	}
}

impl<'a> Announcer for Konmeiko<'a> {
	fn announce(&self, criteria: &AnnouncementCriteria) -> Option<String> {
		debug!("Start building announcement about Konmeiko: {:?}", criteria);

		let monster = self.current_monster(criteria.at());
		match self.is_open(criteria.at()) {
			IsOpen::FirstDay => Some(
				self.announcement_at_start
					.replace("__MONSTERS__", monster.display())
					.replace("__RESISTANCES__", monster.resistances().display(None::<Vec<String>>).as_ref())

			),
			IsOpen::IsInTerm => Some(
				self.announcement
					.replace("__MONSTERS__", monster.display())
			),
			IsOpen::OutOfTerm => None
		}
	}
}

impl<'a> Responder for Konmeiko<'a> {
	fn respond(&self, criteria: &ResponseCriteria) -> Option<String> {
		debug!("start to reaction about Konmeiko: {}", criteria.text());

		if self.is_match(criteria.text()) {
			match self.is_open(criteria.at()) {
				IsOpen::OutOfTerm => {
					Some(self.out_of_term.to_owned())
				},
				_ => {
					Some(self.information
						.replace("__MONSTERS__", self.current_monster(criteria.at()).display())
						.replace("__RESISTANCES__", self.current_monster(criteria.at()).resistances().display(None::<Vec<String>>).as_ref())
					)
				},
			}
		} else {
			None
		}
	}
}

impl<'a> std::ops::Deref for Konmeiko<'a> {
	type Target = KonmeikoJson;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

#[derive(Debug, Clone)]
enum IsOpen {
	FirstDay,
	IsInTerm,
	OutOfTerm,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KonmeikoJson {
	reference_date: DateTime<Local>,
	announcement: String,
	announcement_at_start: String,
	information: String,
	out_of_term: String,
	#[serde(deserialize_with = "transform_string_to_regex")]
	nickname_regex: regex::Regex,
	days: Vec<u32>,
	term_in_hours: i64,
	monsters: Vec<MonsterJson>,
}

#[derive(Debug, Clone)]
struct KonmeikoMonsters<'a> {
	inner: Vec<KonmeikoMonster<'a>>,
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
	#[allow(dead_code)]
	id: String,
	monster: &'a Monster,
}

impl<'a> std::ops::Deref for KonmeikoMonster<'a> {
	type Target = &'a Monster;

	fn deref(&self) -> &Self::Target {
		&self.monster
	}
}

#[derive(Debug, Clone, Deserialize)]
struct MonsterJson {
	id: String,
	monster_id: String,
}
