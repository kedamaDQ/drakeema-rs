use std::fs::File;
use std::io::BufReader;
use chrono::{ DateTime, Local };
use serde::Deserialize;
use crate::{
	Error,
	Result,
	monsters::Monster,
	utils::transform_string_to_regex,
};
use super::{ Responder, ResponseCriteria };

const DATA: &str = "drakeema-data/contents/boueigun.json";

#[derive(Debug, Clone)]
pub struct Boueigun<'a> {
	monsters: BoueigunMonsters<'a>,
	total_duration: i64,
	inner: BoueigunJson,
}

impl<'a> Boueigun<'a> {
	pub fn load() -> Result<Self> {
		info!("Initialize Boueigun");

		let inner: BoueigunJson = serde_json::from_reader(
			BufReader::new(File::open(DATA)?)
		)
		.map_err(|e| Error::UnparseableJson(DATA.to_owned(), e))?;

		let monsters = BoueigunMonsters::new(&inner.monsters)?;
		let total_duration = monsters.iter()
			.fold(0, |acc, m| acc + m.duration);
		Ok(Boueigun {
			monsters,
			total_duration,
			inner,
		})
	}

	fn current_status(&self, at: DateTime<Local>) -> CurrentMonsterInfo {
		let mut ref_date = self.reference_date;

		let monsters = if at < self.reference_date {
			ref_date = ref_date - chrono::Duration::nanoseconds(1);
			self.monsters.iter().rev().collect::<Vec<&BoueigunMonster<'a>>>()
		} else {
			self.monsters.iter().collect::<Vec<&BoueigunMonster<'a>>>()
		};

		let elapsed_min = (at - ref_date).num_minutes().abs();
		let mut elapsed_in_current = elapsed_min % self.total_duration;

		for (index, monster) in monsters.iter().enumerate() {
			if monster.duration > elapsed_in_current {
				let next = if index + 1 < self.monsters.len() {
					self.monsters.get(index + 1).unwrap()
				} else {
					self.monsters.get(0).unwrap()
				};

				return CurrentMonsterInfo{
					current: monster,
					next,
					remain: monster.duration - elapsed_in_current,
				}
			} else {
				elapsed_in_current -= monster.duration;
			}
		}

		panic!("Monster not found: this is a bug");
	}
}

impl<'a> Responder for Boueigun<'a> {
	fn respond(&self, criteria: &ResponseCriteria) -> Option<String> {
		debug!("Start building response about Boueigun: {:?}", criteria);

		if self.nickname_regex.is_match(criteria.text()) {
			info!("Text matched keywords of Boueigun: {}", criteria.text());

			let info = self.current_status(criteria.at());
			let response = self.information
				.replace("__LOCATION__", info.current.location.as_str())
				.replace("__CURRENT_MONSTER__", info.current.display())
				.replace("__RESISTANCES__", info.current.resistances().display(None::<Vec<String>>).as_str())
				.replace("__NEXT_MONSTER__", info.next.display())
				.replace("__REMAIN__", info.remain.to_string().as_str());

			Some(response)
		} else {
			debug!("Nothing response about boueigun: {:?}", criteria);
			None
		}
	}
}

impl<'a> std::ops::Deref for Boueigun<'a> {
	type Target = BoueigunJson;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

#[derive(Debug, Clone)]
struct CurrentMonsterInfo<'a> {
	current: &'a BoueigunMonster<'a>,
	next: &'a BoueigunMonster<'a>,
	remain: i64,
}

#[derive(Debug, Clone)]
struct BoueigunMonsters<'a> {
	inner: Vec<BoueigunMonster<'a>>,
}

impl<'a> std::ops::Deref for BoueigunMonsters<'a> {
	type Target = Vec<BoueigunMonster<'a>>;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl<'a> BoueigunMonsters<'a> {
	fn new(b_monsters: impl AsRef<[MonsterJson]>) -> Result<Self> {
		let mut inner: Vec<BoueigunMonster<'a>> = Vec::new();
		let monsters = crate::monsters();
		for monster in b_monsters.as_ref() {
			match monsters.get(&monster.monster_id) {
				Some(m) => {
					inner.push(BoueigunMonster {
						id: monster.id.to_owned(),
						monster: m,
						location: monster.location.to_owned(),
						duration: monster.duration,
					});
				},
				None => return Err(
					Error::UnknownMonsterId(DATA, monster.id.to_owned())
				),
			};
		}
		Ok(BoueigunMonsters {
			inner,
		})
	}
}

#[derive(Debug, Clone)]
struct BoueigunMonster<'a> {
	#[allow(unused)]
	id: String,
	monster: &'a Monster,
	location: String,
	duration: i64,
}

impl<'a> std::ops::Deref for BoueigunMonster<'a> {
	type Target = Monster;
	fn deref(&self) -> &Self::Target {
		&self.monster
	}
}

#[derive(Debug, Clone, Deserialize)]
pub struct BoueigunJson {
	reference_date: DateTime<Local>,
	information: String,
	#[serde(deserialize_with = "transform_string_to_regex")]
	nickname_regex: regex::Regex,
	monsters: Vec<MonsterJson>,
}

#[derive(Debug, Clone, Deserialize)]
struct MonsterJson {
	id: String,
	monster_id: String,
	location: String,
	duration: i64,
}

#[cfg(test)]
pub(crate) mod tests {
	use super::*;
	use chrono::offset::TimeZone;

	#[test]
	fn test_current_positive() {
		let bou = data();

		assert_eq!(
			bou.current_status(Local.ymd(2020, 6, 5).and_hms(15, 0, 0)).current.id,
			"juga1"
		);

		assert_eq!(
			bou.current_status(Local.ymd(2020, 6, 5).and_hms(15, 59, 59)).current.id,
			"juga1"
		);

		assert_eq!(
			bou.current_status(Local.ymd(2020, 6, 5).and_hms(16, 00, 00)).current.id,
			"tekki1"
		);

		// last of 1st lap
		assert_eq!(
			bou.current_status(bou.reference_date + chrono::Duration::minutes(bou.total_duration - 1)).current.id,
			"all2"
		);
		// 2nd lap
		assert_eq!(
			bou.current_status(bou.reference_date + chrono::Duration::minutes(bou.total_duration)).current.id,
			"juga1"
		);
	}

	#[test]
	fn test_current_negative() {
		let bou = data();

		assert_eq!(
			bou.current_status(Local.ymd(2020, 6, 5).and_hms(14, 59, 59)).current.id,
			"all2"
		);

		assert_eq!(
			bou.current_status(Local.ymd(2020, 6, 5).and_hms(14, 0, 0)).current.id,
			"all2"
		);

		assert_eq!(
			bou.current_status(Local.ymd(2020, 6, 5).and_hms(13, 59, 59)).current.id,
			"ryurin2"
		);
		// last of 1st lap
		assert_eq!(
			bou.current_status(bou.reference_date - chrono::Duration::minutes(bou.total_duration - 1)).current.id,
			"juga1"
		);
		// 2nd lap
		assert_eq!(
			bou.current_status(bou.reference_date - chrono::Duration::minutes(bou.total_duration)).current.id,
			"juga1"
		);
		assert_eq!(
			bou.current_status(bou.reference_date - chrono::Duration::minutes(bou.total_duration) - chrono::Duration::nanoseconds(1)).current.id,
			"all2"
		);

	}

	pub(crate) fn data<'a>() -> Boueigun<'a> {
		let inner: BoueigunJson = serde_json::from_str(DATA).unwrap();
		let monsters = BoueigunMonsters::new(&inner.monsters).unwrap();
		let total_duration = monsters.iter()
			.fold(0, |acc, m| acc + m.duration);
		Boueigun {
			monsters,
			total_duration,
			inner,
		}

	}
	const DATA: &str = r#"{
	"reference_date": "2020-06-05T15:00:00.000+09:00",
	"information": "現在アストルティア防衛軍は __CURRENT_MONSTER__ に攻められています！\n__RESISTANCES__ の耐性があると良いようです！\nあと __REMAIN__ 分で __NEXT_MONSTER__ が攻めてきます！",
	"nickname_regex": "(?:防衛軍|ぼうえいぐん|防衛)",
	"monsters": [
		{ "id": "juga1", "monster_id": "boueigun_juga", "duration": 60 },
		{ "id": "tekki1", "monster_id": "boueigun_tekki", "duration": 60 },
		{ "id": "zoma1", "monster_id": "boueigun_zoma", "duration": 60 },
		{ "id": "ryurin1", "monster_id": "boueigun_ryurin", "duration": 60 },
		{ "id": "all1", "monster_id": "boueigun_all", "duration": 60 },
		{ "id": "shigoku1", "monster_id": "boueigun_shigoku", "duration": 60 },
		{ "id": "kyochu1", "monster_id": "boueigun_kyochu", "duration": 60 },
		{ "id": "kaiyo1", "monster_id": "boueigun_kaiyo", "duration": 60 },
		{ "id": "ryurin2", "monster_id": "boueigun_ryurin", "duration": 60 },
		{ "id": "all2", "monster_id": "boueigun_all", "duration": 60 }
	]
	}"#;
}
