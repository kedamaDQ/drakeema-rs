use std::convert::TryFrom;
use std::fs::File;
use std::io::BufReader;
use chrono::{ DateTime, Duration, Local, };
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

const DATA: &str = "drakeema-data/contents/panigulm.json";

#[derive(Debug, Clone)]
pub struct Panigulm<'a> {
	monsters: Vec<PanigulmMonster<'a>>,
	inner: PanigulmJson,
}

impl<'a> Panigulm<'a> {
	pub fn load() -> Result<Self> {
		info!("Initialize Panigulm");

		let inner: PanigulmJson = serde_json::from_reader(
			BufReader::new(File::open(DATA)?)
		)
		.map_err(|e| Error::UnparseableJson(DATA.to_owned(), e))?;

		let mut monsters: Vec<PanigulmMonster<'a>> = Vec::new();

		for monster_id in &inner.monster_ids {
			match crate::monsters().get(monster_id) {
				Some(monster) => monsters.push(PanigulmMonster{ monster }),
				None => return Err(
					Error::UnknownMonsterId(DATA, monster_id.to_owned())
				)
			}
		}

		Ok(Panigulm {
			monsters,
			inner,
		})
	}

	fn monster_at(&self, at: DateTime<Local>) -> &PanigulmMonster {
		let mut ref_date = self.reference_date;

		let monsters: Vec<&PanigulmMonster<'a>> = if at < self.reference_date {
			ref_date = ref_date - Duration::nanoseconds(1);
			self.monsters.iter().rev().collect()
		} else {
			self.monsters.iter().collect()
		};

		let current_index: usize = usize::try_from(
			(at - ref_date).num_days().abs() / self.num_days % i64::try_from(monsters.len()).unwrap()
		).unwrap();

		monsters.get(current_index).unwrap()
	}
}

impl<'a> Announcer for Panigulm<'a> {
	fn announce(&self, criteria: &AnnouncementCriteria) -> Option<String> {
		debug!("Start building announcement about Panigulm: {:?}", criteria);

		let monster_today = self.monster_at(criteria.at());
		let monster_tomorrow = self.monster_at(criteria.at() + Duration::days(1));
		let monster_yesterday = self.monster_at(criteria.at() + Duration::days(-1));

		let announcement = if monster_today != monster_yesterday {
			self.announcement_at_start
				.replace("__MONSTER__", monster_today.display())
				.replace("__RESISTANCES__", monster_today.resistances().display(None::<Vec<String>>).as_str())
		} else if monster_today != monster_tomorrow {
			self.announcement_at_end
				.replace("__MONSTER1__", monster_today.display())
				.replace("__MONSTER2__", monster_tomorrow.display())
		} else {
			self.announcement
				.replace("__MONSTER__", monster_today.display())
		};

		Some(announcement)
	}
}

impl<'a> Responder for Panigulm<'a> {
	fn respond(&self, criteria: &ResponseCriteria) -> Option<String> {
		debug!("Start building response about Panigulm: {:?}", criteria);

		if self.nickname_regex.is_match(criteria.text()) {
			info!("Text matches some keywords of Panigulm: {}", criteria.text());
			let monster = self.monster_at(criteria.at());
			let response = self.information
				.replace("__MONSTER__", monster.display())
				.replace("__RESISTANCES__", monster.resistances().display(None::<Vec<String>>).as_str());
			Some(response)
		} else {
			debug!("Text unmatched any keywords of Panigulm: {:?}", criteria);
			None
		}
	}
}

#[derive(Debug, Clone)]
struct PanigulmMonster<'a> {
	monster: &'a Monster,
}

impl<'a> std::ops::Deref for PanigulmMonster<'a> {
	type Target = Monster;

	fn deref(&self) -> &Self::Target {
		self.monster
	}
}

use std::cmp;

impl<'a> cmp::PartialEq for PanigulmMonster<'a> {
	fn eq(&self, other: &Self) -> bool {
		self.id() == other.id()
	}
}

impl<'a> cmp::PartialOrd for PanigulmMonster<'a> {
	fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl<'a> cmp::Eq for PanigulmMonster<'a> {}

impl<'a> cmp::Ord for PanigulmMonster<'a> {
	fn cmp(&self, other: &Self) -> cmp::Ordering {
		self.id().cmp(other.id())
	}
}

#[derive(Debug, Clone, Deserialize)]
pub struct PanigulmJson {
	reference_date: DateTime<Local>,
	num_days: i64,
	announcement: String,
	announcement_at_start: String,
	announcement_at_end: String,
	information: String,
	#[serde(deserialize_with = "transform_string_to_regex")]
	nickname_regex: regex::Regex,
	monster_ids: Vec<String>,
}

impl<'a> std::ops::Deref for Panigulm<'a> {
	type Target = PanigulmJson;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}


#[cfg(test)]
pub(crate) mod tests {
	use super::*;
	use chrono::offset::TimeZone;

	#[test]
	fn test_current_positive() {
		let pani = data();

		assert_eq!(
			pani.monster_at(Local.with_ymd_and_hms(2025, 2, 1, 6, 0, 0).unwrap()).id(),
			"panigulm_jigenryu"
		);

		assert_eq!(
			pani.monster_at(Local.with_ymd_and_hms(2025, 2, 4, 5, 59, 59).unwrap()).id(),
			"panigulm_jigenryu"
		);

		assert_eq!(
			pani.monster_at(Local.with_ymd_and_hms(2025, 2, 4, 6, 0, 0).unwrap()).id(),
			"panigulm_fordina"
		);

		assert_eq!(
			pani.monster_at(Local.with_ymd_and_hms(2025, 2, 7, 5, 59, 59).unwrap()).id(),
			"panigulm_fordina"
		);

		assert_eq!(
			pani.monster_at(Local.with_ymd_and_hms(2025, 2, 7, 6, 0, 0).unwrap()).id(),
			"panigulm_dydalmos"
		);

		assert_eq!(
			pani.monster_at(Local.with_ymd_and_hms(2025, 2, 25, 5, 59, 59).unwrap()).id(),
			"panigulm_almana"
		);

		assert_eq!(
			pani.monster_at(Local.with_ymd_and_hms(2025, 2, 25, 6, 0, 0).unwrap()).id(),
			"panigulm_jigenryu"
		);

		// run `cargo test -- --nocapture`
		let ac = AnnouncementCriteria {
			at: Local.with_ymd_and_hms(2025, 2, 1, 6, 0, 0).unwrap(),
		};
		println!("Announce: {:#?}", pani.announce(&ac));

		let ac = AnnouncementCriteria {
			at: Local.with_ymd_and_hms(2022, 2, 4, 6, 0, 0).unwrap(),
		};
		println!("Announce: {:#?}", pani.announce(&ac));

		let ac = AnnouncementCriteria {
			at: Local.with_ymd_and_hms(2022, 2, 24, 5, 59, 59).unwrap(),
		};
		println!("Announce: {:#?}", pani.announce(&ac));
	}

	#[test]
	fn test_current_negative() {
		let pani = data();

		assert_eq!(
			pani.monster_at(Local.with_ymd_and_hms(2025, 2, 1, 5, 59, 59).unwrap()).id(),
			"panigulm_almana"
		);

		assert_eq!(
			pani.monster_at(Local.with_ymd_and_hms(2025, 1, 29, 6, 0, 0).unwrap()).id(),
			"panigulm_almana"
		);

		assert_eq!(
			pani.monster_at(Local.with_ymd_and_hms(2025, 1, 29, 5, 59, 59).unwrap()).id(),
			"panigulm_elgios"
		);

		assert_eq!(
			pani.monster_at(Local.with_ymd_and_hms(2025, 1, 26, 6, 0, 0).unwrap()).id(),
			"panigulm_elgios"
		);

		assert_eq!(
			pani.monster_at(Local.with_ymd_and_hms(2025, 1, 10, 5, 59, 59).unwrap()).id(),
			"panigulm_jigenryu"
		);

		assert_eq!(
			pani.monster_at(Local.with_ymd_and_hms(2025, 1, 8, 6, 0, 0).unwrap()).id(),
			"panigulm_jigenryu"
		);

		assert_eq!(
			pani.monster_at(Local.with_ymd_and_hms(2025, 1, 8, 5, 59, 59).unwrap()).id(),
			"panigulm_almana"
		);
	}

	#[test]
	fn test_respond() {
		let pani = data();

		let rc = ResponseCriteria { text: "あああパニパニあああ".to_owned(), at: Local.with_ymd_and_hms(2022, 2, 22, 7, 0, 0).unwrap() };
		assert!(pani.respond(&rc).is_some());

		let rc = ResponseCriteria { text: "源世庫".to_owned(), at: Local.with_ymd_and_hms(2022, 2, 22, 7, 0, 0).unwrap() };
		assert!(pani.respond(&rc).is_some());

		let rc = ResponseCriteria { text: "パニゴロモ".to_owned(), at: Local.with_ymd_and_hms(2022, 2, 22, 7, 0, 0).unwrap() };
		assert!(pani.respond(&rc).is_none());

		let rc = ResponseCriteria { text: "源世庫".to_owned(), at: Local.with_ymd_and_hms(2022, 2, 22, 7, 0, 0).unwrap() };
		println!("Response: {:?}", pani.respond(&rc));
	}

	pub(crate) fn data<'a>() -> Panigulm<'a> {
		let inner: PanigulmJson = serde_json::from_str(DATA).unwrap();
		let mut monsters: Vec<PanigulmMonster<'a>> = Vec::new();

		for monster_id in &inner.monster_ids {
			monsters.push(PanigulmMonster {
				monster: crate::monsters().get(monster_id).unwrap()
			});
		}

		Panigulm {
			inner,
			monsters,
		}
	}

	const DATA: &str = r#" {
	"reference_date": "2025-02-01T06:00:00.000+09:00",
	"num_days": 3,
	"announcement": "本日の源世庫パニガルムは __MONSTER__ です！",
	"announcement_at_start": "源世庫パニガルムは本日から __MONSTER__、あると良い耐性は __RESISTANCES__ です！",
	"announcement_at_end": "本日の源世庫パニガルムは __MONSTER1__ です！明日からは __MONSTER2__ が始まります！",
	"information": "本日の源世庫パニガルムは __MONSTER__ です！あると良い耐性は __RESISTANCES__ です！",
	"nickname_regex": "(?:(?:源世庫|げんせいこ|ゲンセイコ|ｹﾞﾝｾｲｺ)|(?:パニガルム|ぱにがるむ|ﾊﾟﾆｶﾞﾙﾑ|パニパニ|ぱにぱに|ﾊﾟﾆﾊﾟﾆ))",
	"monster_ids": [
		"panigulm_jigenryu",
		"panigulm_fordina",
		"panigulm_dydalmos",
		"panigulm_catcher",
		"panigulm_fulupotea",
		"panigulm_pultanus",
		"panigulm_elgios",
		"panigulm_almana"
	]
	}"#;
}
