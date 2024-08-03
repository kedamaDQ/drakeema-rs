use std::fs::File;
use std::io::BufReader;
use chrono:: {
	Datelike,
	DateTime,
	Duration,
	Local,
	LocalResult,
	Timelike,
	TimeZone
};
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

	fn event_status(&self, at: DateTime<Local>) -> EventStatus {
		let term_in_hours: Duration = Duration::hours(self.term_in_hours);

		for d in self.days.iter() {
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
					return EventStatus::OutOfTerm;
				},

			};

			let term_end: DateTime<Local> = match term_start.checked_add_signed(term_in_hours) {
				Some(te) => te,
				None => {
					error!("DateTime addition failed");
					return EventStatus::OutOfTerm;
				},
			};

			if &at.day() == d {
				return EventStatus::StartOfTerm{
					start: term_start,
					end: term_end
				};
			}

			if term_start < at && at < term_end {
				return EventStatus::OnTerm {
					start: term_start.to_owned(),
					end: term_end.to_owned(),
				};
			}
		}

		EventStatus::OutOfTerm
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
				(12 - self.reference_date.month0()) * 2 +
				// number of past year's switching
				(at.year() - self.reference_date.year() - 1) as u32 * 2 +
				// number of this year's switching
				(at.month() - 1) * 2 +
				// number of this month's switching
				at.day() / 15
			) as usize % self.monsters.len() as usize
		};

		self.monsters.get(index).unwrap()
	}
}

impl<'a> Announcer for Konmeiko<'a> {
	fn announce(&self, criteria: &AnnouncementCriteria) -> Option<String> {
		debug!("Start building announcement about Konmeiko: {:?}", criteria);

		let monster = self.current_monster(criteria.at());
		match self.event_status(criteria.at()) {
			EventStatus::StartOfTerm{ start: _, end } => Some(
				self.announcement_at_start
					.replace("__MONSTERS__", monster.display())
					.replace("__RESISTANCES__", monster.resistances().display(None::<Vec<String>>).as_ref())
					.replace("__END_OF_TERM__", format!(
						"{}年{}月{}日の{}時", end.year(), end.month(), end.day(), end.hour()
					).as_str())

			),
			EventStatus::OnTerm{ start: _, end } => Some(
				self.announcement
					.replace("__MONSTERS__", monster.display())
					.replace( "__END_OF_TERM__", format!(
						"{}年{}月{}日の{}時", end.year(), end.month(), end.day(), end.hour()
					).as_str())
			),
			EventStatus::OutOfTerm => None
		}
	}
}

impl<'a> Responder for Konmeiko<'a> {
	fn respond(&self, criteria: &ResponseCriteria) -> Option<String> {
		debug!("start to reaction about Konmeiko: {}", criteria.text());

		if self.is_match(criteria.text()) {
			match self.event_status(criteria.at()) {
				EventStatus::OutOfTerm => {
					Some(self.out_of_term.to_owned())
				},
				EventStatus::StartOfTerm {start: _, end } |
				EventStatus::OnTerm {start: _, end } =>
					Some(self.information
						.replace("__MONSTERS__", self.current_monster(criteria.at()).display())
						.replace("__RESISTANCES__", self.current_monster(criteria.at()).resistances().display(None::<Vec<String>>).as_ref())
						.replace( "__END_OF_TERM__", format!(
							"{}年{}月{}日の{}時", end.year(), end.month(), end.day(), end.hour()
						).as_str())
					)
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

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
enum EventStatus {
	StartOfTerm {
		start: DateTime<Local>,
		end: DateTime<Local>
	},
	OnTerm {
		start: DateTime<Local>,
		end: DateTime<Local>
	},
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

impl<'a> KonmeikoMonster<'a> {
	#[allow(dead_code)]
	pub fn id(&self) -> &str {
		&self.id
	}

	#[allow(dead_code)]
	pub fn monster(&self) -> &'a Monster {
		&self.monster
	}
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

#[cfg(test)]
pub(crate) mod tests {
	use super::*;
	use chrono::Local;

	#[test]
	fn test_event_status() {
		let kmk = data(1);

		assert_eq!(
			kmk.event_status(Local.with_ymd_and_hms(
				2024, 07, 01,
				6, 0, 0).unwrap()
			), EventStatus::StartOfTerm {
				start: Local.with_ymd_and_hms(
					2024, 07, 01,
					6, 0, 0).unwrap(),
				end: Local.with_ymd_and_hms(
					2024, 07, 06,
					6, 0, 0).unwrap(),
			}
		);

		assert_eq!(
			kmk.event_status(Local.with_ymd_and_hms(
				2024, 07, 02,
				9, 0, 0).unwrap()
			), EventStatus::OnTerm {
				start: Local.with_ymd_and_hms(
					2024, 07, 01,
					6, 0, 0).unwrap(),
				end: Local.with_ymd_and_hms(
					2024, 07, 06,
					6, 0, 0).unwrap(),
			}
		);
		
		assert_eq!(
			kmk.event_status(Local.with_ymd_and_hms(
				2024, 07, 06,
				5, 59, 59).unwrap()
			), EventStatus::OnTerm {
				start: Local.with_ymd_and_hms(
					2024, 07, 01,
					6, 0, 0).unwrap(),
				end: Local.with_ymd_and_hms(
					2024, 07, 06,
					6, 0, 0).unwrap(),
			}
		);
		
		assert_eq!(
			kmk.event_status(Local.with_ymd_and_hms(
				2024, 07, 06,
				6, 0, 0).unwrap()
			), EventStatus::OutOfTerm
		);

		assert_eq!(
			kmk.event_status(Local.with_ymd_and_hms(
				2024, 07, 10,
				13, 0, 0).unwrap()
			), EventStatus::OutOfTerm
		);

		assert_eq!(
			kmk.event_status(Local.with_ymd_and_hms(
				2024, 07, 15,
				6, 0, 0).unwrap()
			), EventStatus::StartOfTerm {
				start: Local.with_ymd_and_hms(
					2024, 07, 15,
					6, 0, 0).unwrap(),
				end: Local.with_ymd_and_hms(
					2024, 07, 20,
					6, 0, 0).unwrap(),
			}
		);

		assert_eq!(
			kmk.event_status(Local.with_ymd_and_hms(
				2024, 07, 16,
				9, 0, 0).unwrap()
			), EventStatus::OnTerm {
				start: Local.with_ymd_and_hms(
					2024, 07, 15,
					6, 0, 0).unwrap(),
				end: Local.with_ymd_and_hms(
					2024, 07, 20,
					6, 0, 0).unwrap(),
			}
		);
		
		assert_eq!(
			kmk.event_status(Local.with_ymd_and_hms(
				2024, 07, 20,
				5, 59, 59).unwrap()
			), EventStatus::OnTerm {
				start: Local.with_ymd_and_hms(
					2024, 07, 15,
					6, 0, 0).unwrap(),
				end: Local.with_ymd_and_hms(
					2024, 07, 20,
					6, 0, 0).unwrap(),
			}
		);
		
		assert_eq!(
			kmk.event_status(Local.with_ymd_and_hms(
				2024, 07, 20,
				6, 0, 0).unwrap()
			), EventStatus::OutOfTerm
		);
	}

	#[test]
	fn test_monster_lotation() {
		let kmk = data(3);

		assert_eq!(
			kmk.current_monster(Local.with_ymd_and_hms(
				2024, 07, 1,
				6, 0, 0).unwrap()).id()
			, "test1"
		);

		assert_eq!(
			kmk.current_monster(Local.with_ymd_and_hms(
				2024, 07, 15,
				6, 0, 0).unwrap()).id()
			, "test2"
		);

		assert_eq!(
			kmk.current_monster(Local.with_ymd_and_hms(
				2024, 08, 1,
				6, 0, 0).unwrap()).id()
			, "test3"
		);

		assert_eq!(
			kmk.current_monster(Local.with_ymd_and_hms(
				2024, 08, 15,
				6, 0, 0).unwrap()).id()
			, "test1"
		);

		// 6 months * 2 + 1
		assert_eq!(
			kmk.current_monster(Local.with_ymd_and_hms(
				2025, 1, 1,
				6, 0, 0).unwrap()).id()
			, "test1"
		);

		// 8 months * 2 + 1
		assert_eq!(
			kmk.current_monster(Local.with_ymd_and_hms(
				2025, 3, 1,
				6, 0, 0).unwrap()).id()
			, "test2"
		);

		// (12 months + 5 months) * 2 + 1
		// 35 mod 3: 2
		assert_eq!(
			kmk.current_monster(Local.with_ymd_and_hms(
				2026, 4, 1,
				6, 0, 0).unwrap()).id()
			, "test3"
		);
	}

	#[test]
	fn test_messages() {
		let kmk: Konmeiko = data(1);

		assert_eq!(
			kmk.announce(&AnnouncementCriteria::new(Local.with_ymd_and_hms(
				2024, 7, 1,
				6, 0, 0).unwrap())).unwrap(),
			"昏冥庫パニガルムが開放されました！2024年7月6日の6時まで 冥氷竜ジェロドーラ と戦えます！呪文、おびえ、氷、闇の耐性があると良いようです！".to_string()
		);

		assert_eq!(
			kmk.announce(&AnnouncementCriteria::new(Local.with_ymd_and_hms(
				2024, 7, 6,
				5, 59, 59).unwrap())).unwrap(),
			"本日の昏冥庫パニガルムは 冥氷竜ジェロドーラ です！2024年7月6日の6時まで開放されています！",
		);

		assert_eq!(
			kmk.announce(&AnnouncementCriteria::new(Local.with_ymd_and_hms(
				2024, 7, 6,
				6, 0, 0).unwrap())),
			None,
		);

		assert_eq!(
			kmk.respond(&ResponseCriteria::new(Local.with_ymd_and_hms(
				2024, 7, 15,
				6, 0, 0).unwrap(), "こんめーこ")).unwrap(),
			"本日の昏冥庫パニガルムは 冥氷竜ジェロドーラ です！2024年7月20日の6時まで開放されています！呪文、おびえ、氷、闇の耐性があると良いようです！",
		);

		assert_eq!(
			kmk.respond(&ResponseCriteria::new(Local.with_ymd_and_hms(
				2024, 7, 20,
				6, 0, 0).unwrap(), "こんめーこ")).unwrap(),
			"本日の昏冥庫パニガルムは開いてません！",
		);
	}

	pub(crate) fn data<'a>(num: u8) -> Konmeiko<'a> {
		let inner: &str = match num {
			1 => TEST_DATA1,
			2 => TEST_DATA2,
			3 => TEST_DATA3,
			_ => panic!("hoge"),
		};

		let inner: KonmeikoJson = serde_json::from_str(inner).unwrap();
		Konmeiko {
			monsters: KonmeikoMonsters::new(&inner.monsters).unwrap(),
			inner,
		}
	}

	const TEST_DATA1: &str = r#"
		{
			"attention": "!! set first day to reference_date !!",
			"reference_date": "2024-07-01T06:00:00.000+09:00",
			"announcement": "本日の昏冥庫パニガルムは __MONSTERS__ です！__END_OF_TERM__まで開放されています！",
			"announcement_at_start": "昏冥庫パニガルムが開放されました！__END_OF_TERM__まで __MONSTERS__ と戦えます！__RESISTANCES__の耐性があると良いようです！",
			"information": "本日の昏冥庫パニガルムは __MONSTERS__ です！__END_OF_TERM__まで開放されています！__RESISTANCES__の耐性があると良いようです！",
			"out_of_term": "本日の昏冥庫パニガルムは開いてません！",
			"nickname_regex": "(?:昏冥庫|こんめいこ|こんめーこ|混迷庫)",
			"days": [1, 15],
			"term_in_hours": 120,
			"monsters": [
		   		{
		   			"id": "jerodra",
					"monster_id": "konmeiko_jerodra"
				}
			]
		}
	"#;

	const TEST_DATA2: &str = r#"
		{
			"attention": "!! set first day to reference_date !!",
			"reference_date": "2024-07-01T06:00:00.000+09:00",
			"announcement": "本日の昏冥庫パニガルムは __MONSTERS__ です！__END_OF_TERM__まで開放されています！",
			"announcement_at_start": "昏冥庫パニガルムが開放されました！__END_OF_TERM__まで __MONSTERS__ と戦えます！__RESISTANCES__の耐性があると良いようです！",
			"information": "本日の昏冥庫パニガルムは __MONSTERS__ です！__END_OF_TERM__まで開放されています！__RESISTANCES__の耐性があると良いようです！",
			"out_of_term": "本日の昏冥庫パニガルムは開いてません！",
			"nickname_regex": "(?:昏冥庫|こんめいこ|こんめーこ|混迷庫)",
			"days": [1, 15],
			"term_in_hours": 120,
			"monsters": [
		   		{
		   			"id": "test1",
					"monster_id": "konmeiko_jerodra"
				},
		   		{
		   			"id": "test2",
					"monster_id": "konmeiko_jerodra"
				}
			]
		}
	"#;

	const TEST_DATA3: &str = r#"
		{
			"attention": "!! set first day to reference_date !!",
			"reference_date": "2024-07-01T06:00:00.000+09:00",
			"announcement": "本日の昏冥庫パニガルムは __MONSTERS__ です！__END_OF_TERM__まで開放されています！",
			"announcement_at_start": "昏冥庫パニガルムが開放されました！__END_OF_TERM__まで __MONSTERS__ と戦えます！__RESISTANCES__の耐性があると良いようです！",
			"information": "本日の昏冥庫パニガルムは __MONSTERS__ です！__END_OF_TERM__まで開放されています！__RESISTANCES__の耐性があると良いようです！",
			"out_of_term": "本日の昏冥庫パニガルムは開いてません！",
			"nickname_regex": "(?:昏冥庫|こんめいこ|こんめーこ|混迷庫)",
			"days": [1, 15],
			"term_in_hours": 120,
			"monsters": [
		   		{
		   			"id": "test1",
					"monster_id": "konmeiko_jerodra"
				},
		   		{
		   			"id": "test2",
					"monster_id": "konmeiko_jerodra"
				},
		   		{
		   			"id": "test3",
					"monster_id": "konmeiko_jerodra"
				}
			]
		}
	"#;
}