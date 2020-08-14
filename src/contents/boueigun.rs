use std::fs::File;
use std::io::BufReader;
use chrono::{ DateTime, Local };
use serde::Deserialize;
use crate::{
	Error,
	Result,
	monsters::{ Monster, Monsters },
	utils::transform_string_to_regex,
};
use super::{ Information, InformationCriteria };

const DATA: &str = "data/contents/boueigun.json";

pub struct Boueigun<'a> {
	monsters: BoueigunMonsters<'a>,
	total_duration: i64,
	inner: BoueigunJson,
}

impl<'a> Boueigun<'a> {
	pub fn load(monsters: &'a Monsters) -> Result<Self> {
		let inner: BoueigunJson = match serde_json::from_reader(
			BufReader::new(File::open(DATA)?)
		) {
			Ok(bj) => bj,
			Err(e) => return Err(
				Error::ParseJsonError(DATA.to_owned(), e)
			)
		};

		let monsters = BoueigunMonsters::new(&inner.monsters, monsters)?;
		let total_duration = monsters.iter()
			.fold(0, |acc, m| acc + m.duration);
		Ok(Boueigun {
			monsters,
			total_duration,
			inner,
		})
	}

	fn current_monster(&self, at: DateTime<Local>) -> CurrentMonsterInfo {
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

impl<'a> Information for Boueigun<'a> {
	fn information(&self, criteria: InformationCriteria) -> Option<String> {
		if self.nickname_regex.is_match(&criteria.text) {
			let info = self.current_monster(criteria.at);
			Some(self.information
				.replace("__CURRENT_MONSTER__", info.current.display())
				.replace("__RESISTANCES__", info.current.resistances().display(None::<Vec<String>>).as_str())
				.replace("__NEXT_MONSTER__", info.next.display())
				.replace("__REMAIN__", info.remain.to_string().as_str())
			)
		} else {
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

struct CurrentMonsterInfo<'a> {
	current: &'a BoueigunMonster<'a>,
	next: &'a BoueigunMonster<'a>,
	remain: i64,
}

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
	fn new(b_monsters: impl AsRef<[MonsterJson]>, monsters: &'a Monsters) -> Result<Self> {
		let mut inner: Vec<BoueigunMonster<'a>> = Vec::new();
		for monster in b_monsters.as_ref() {
			match monsters.get(&monster.monster_id) {
				Some(m) => {
					inner.push(BoueigunMonster {
						id: monster.id.to_owned(),
						monster: m,
						duration: monster.duration,
					});
				},
				None => return Err(
					Error::UnknownMonsterIdError(DATA, monster.id.to_owned())
				),
			};
		}
		Ok(BoueigunMonsters {
			inner,
		})
	}
}

struct BoueigunMonster<'a> {
	#[allow(unused)]
	id: String,
	monster: &'a Monster,
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
	duration: i64,
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::monsters;
	use chrono::offset::TimeZone;

	#[test]
	fn test_current_positive() {
		let mon = monsters::load().unwrap();
		let bou = load(&mon);

		assert_eq!(
			bou.current_monster(Local.ymd(2020, 6, 5).and_hms(15, 0, 0)).current.id,
			"juga1"
		);

		assert_eq!(
			bou.current_monster(Local.ymd(2020, 6, 5).and_hms(15, 59, 59)).current.id,
			"juga1"
		);

		assert_eq!(
			bou.current_monster(Local.ymd(2020, 6, 5).and_hms(16, 00, 00)).current.id,
			"tekki1"
		);

		// last of 1st lap
		assert_eq!(
			bou.current_monster(bou.reference_date + chrono::Duration::minutes(bou.total_duration - 1)).current.id,
			"all2"
		);
		// 2nd lap
		assert_eq!(
			bou.current_monster(bou.reference_date + chrono::Duration::minutes(bou.total_duration)).current.id,
			"juga1"
		);
	}

	#[test]
	fn test_current_negative() {
		let mon = monsters::load().unwrap();
		let bou = load(&mon);

		assert_eq!(
			bou.current_monster(Local.ymd(2020, 6, 5).and_hms(14, 59, 59)).current.id,
			"all2"
		);

		assert_eq!(
			bou.current_monster(Local.ymd(2020, 6, 5).and_hms(14, 0, 0)).current.id,
			"all2"
		);

		assert_eq!(
			bou.current_monster(Local.ymd(2020, 6, 5).and_hms(13, 59, 59)).current.id,
			"ryurin2"
		);
		// last of 1st lap
		assert_eq!(
			bou.current_monster(bou.reference_date - chrono::Duration::minutes(bou.total_duration - 1)).current.id,
			"juga1"
		);
		// 2nd lap
		assert_eq!(
			bou.current_monster(bou.reference_date - chrono::Duration::minutes(bou.total_duration)).current.id,
			"juga1"
		);
		assert_eq!(
			bou.current_monster(bou.reference_date - chrono::Duration::minutes(bou.total_duration) - chrono::Duration::nanoseconds(1)).current.id,
			"all2"
		);

	}

	fn load(monsters: &Monsters) -> Boueigun {
		let inner: BoueigunJson = serde_json::from_str(DATA).unwrap();
		let monsters = BoueigunMonsters::new(&inner.monsters, monsters).unwrap();
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
