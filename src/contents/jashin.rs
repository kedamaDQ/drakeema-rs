use std::fs::File;
use std::io::BufReader;
use chrono::{ Datelike, DateTime, Duration, Local };
use serde::Deserialize;
use crate::{
	Error,
	Result,
	monsters::Monster,
	resistances::Resistances,
	utils::transform_string_to_regex,
};
use super::{
	Announcer,
	AnnouncementCriteria,
	Responder,
	ResponseCriteria,
};

const DATA: &str = "drakeema-data/contents/jashin.json";

#[derive(Debug, Clone)]
pub struct Jashin<'a> {
	tables: Tables<'a>,
	inner: JashinJson,
}

impl<'a> Jashin<'a> {
	pub fn load() -> Result<Self> {
		info!("Initialize Jashin");

    	let mut inner: JashinJson = serde_json::from_reader(
    		BufReader::new(File::open(DATA)?)
		)
		.map_err(|e| Error::UnparseableJson(DATA.to_owned(), e))?;

		inner.tables.sort_by(|a, b| a.start_day.cmp(&b.start_day));

		Ok(Jashin {
			tables: Tables::new(&inner.tables, inner.reference_date)?,
			inner,
		})
	}

	fn title(&self, at: DateTime<Local>) -> &Title {
		self.tables.table(at).titles.title(at)
	}
}

impl<'a> Announcer for Jashin<'a> {
	fn announce(&self, criteria: &AnnouncementCriteria) -> Option<String> {
		use std::ops::Add;

		debug!("Start building announce about Jashin: {:?}", criteria);

		let title_today = self.title(criteria.at());
		let title_tomorrow = self.title(criteria.at().add(Duration::hours(24)));
		let title_yesterday = self.title(criteria.at().add(Duration::hours(-24)));

		let announcement = if title_today != title_yesterday {
			// Date is start date of the period
			self.announcement_at_start
				.replace("__TITLE__", title_today.display_title())
				.replace("__MONSTERS__", title_today.display_monsters().as_str())
				.replace("__RESISTANCES__", title_today.display_resistances(Some(&self.area_names)).as_str())
		} else if title_today != title_tomorrow {
			// Date is end date of period
			self.announcement_at_end
				.replace("__TITLE1__", title_today.display_title())
				.replace("__TITLE2__", title_tomorrow.display_title())
		} else {
			// Date is duaring the period
			self.announcement
				.replace("__TITLE__", title_today.display_title())
		};

		Some(announcement)
	}
}

impl<'a> Responder for Jashin<'a> {
	fn respond(&self, criteria: &ResponseCriteria) -> Option<String> {
		debug!("Start building response about Jashin: {:?}", criteria);

		if self.nickname_regex.is_match(criteria.text()) {
			info!("Text matched keywords of Jashin: {}", criteria.text());

			let title = self.title(criteria.at());
			let response = self.information
    			.replace("__TITLE__", title.display_title())
    			.replace("__MONSTERS__", title.display_monsters().as_str())
				.replace("__RESISTANCES__", title.display_resistances(Some(&self.area_names)).as_str());
			
			Some(response)
		} else {
			debug!("Nothing response about jashin: {:?}", criteria);
			None
		}
	}
}

impl<'a> std::ops::Deref for Jashin<'a> {
	type Target = JashinJson;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

#[derive(Debug, Clone)]
struct Tables<'a> {
	reference_date: DateTime<Local>,
	inner: Vec<Table<'a>>,
}

impl<'a> Tables<'a> {
	fn new(
		tables: impl AsRef<[TableJson]>,
		reference_date: DateTime<Local>) -> Result<Self> {
		let mut inner: Vec<Table> = Vec::new();

		for table in tables.as_ref() {
			inner.push(Table {
				start_day: table.start_day,
				titles: Titles::new(&table.titles, reference_date)?
			})
		}

		if inner.is_empty() {
			Err(Error::DataNotPresented(DATA, "element of tables".to_owned()))
		} else {
			Ok(Tables {
				reference_date,
				inner
			})
		}
	}

	fn table(&self, at: DateTime<Local>) -> &Table {
		self.iter()
			.rev()
			.find(|table| {
				at.day() > table.start_day ||
				(at.day() == table.start_day && at.time() >= self.reference_date.time())
			})
			.unwrap_or_else(||
				// Safe unwrapping because constructer new() guarantees inner is not empty.
				self.last().unwrap()
			)
	}
}

impl<'a> std::ops::Deref for Tables<'a> {
	type Target = Vec<Table<'a>>;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

#[derive(Debug, Clone)]
struct Table<'a> {
	start_day: u32,
	titles: Titles<'a>
}

#[derive(Debug, Clone)]
struct Titles<'a> {
	reference_date: DateTime<Local>,
	inner: Vec<Title<'a>>,
}

impl<'a> Titles<'a> {
	fn new(
		titles: impl AsRef<[TitleJson]>,
		reference_date: DateTime<Local>
	) -> Result<Self> {
		let mut inner: Vec<Title<'a>> = Vec::new();
		let monsters = crate::monsters();

		for title in titles.as_ref() {
			let mut mon: Vec<&'a Monster> = Vec::new();

			for monster_id in title.monster_ids.iter() {
				match monsters.get(monster_id) {
					Some(monster) => mon.push(monster),
					None => return Err(
						Error::UnknownMonsterId(DATA, monster_id.to_owned())
					)
				}
			}

			inner.push(Title {
				id: title.id.clone(),
				display: title.display.clone(),
				monsters: mon,
			});
		}

		if inner.is_empty() {
			Err(
				Error::DataNotPresented(DATA, "element of titles".to_owned())
			)
		} else {
			Ok(Titles {
				reference_date,
				inner,
			})
		}
	}

	fn title(&self, at: DateTime<Local>) -> &Title {
		let elapsed_months = self.elapsed_months(at);
		let elapsed_months = if elapsed_months < 0i32 {
			let len = self.len() as i32;
			(len + elapsed_months % len).abs() as usize
		} else {
			elapsed_months as usize % self.len()
		};

		// Safe unwrapping because constructer new() guarantees inner is not empty.
		self.get(elapsed_months).unwrap()
	}

	fn elapsed_months(&self, at: DateTime<Local>) -> i32 {
		let mut elapsed_months =
			(at.year() - self.reference_date.year()) * 12 +
			(at.month() as i32 - self.reference_date.month() as i32);

		if at.day() < self.reference_date.day() || (
			at.day() == self.reference_date.day() &&
			at.time() < self.reference_date.time()
		) {
			elapsed_months -= 1;
		}

		debug!(
			"Number of months elapsed from {} to {} is {}",
			self.reference_date,
			at,
			elapsed_months
		);

		elapsed_months
	}
}

impl<'a> std::ops::Deref for Titles<'a> {
	type Target = Vec<Title<'a>>;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

#[derive(Debug, Clone)]
struct Title<'a> {
	id: String,
	display: String,
	monsters: Vec<&'a Monster>,
}

impl<'a> Title<'a> {
	fn display_title(&self) -> &str {
		self.display.as_str()
	}

	fn display_monsters(&self) -> String {
		self.monsters.iter()
			.map(|m| m.display())
			.collect::<Vec<&str>>()
			.join("と")
	}

	fn display_resistances<T, U>(&self, area_names: Option<T>) -> String
	where
		T: AsRef<[U]>,
		U: AsRef<str>
	{
		self.monsters.iter()
			.map(|m| m.resistances())
			.fold(Resistances::new(), |acc, r| acc.join(r))
			.display(area_names)
	}
}

use std::cmp;

impl<'a> cmp::PartialEq for Title<'a> {
	fn eq(&self, other: &Self) -> bool {
		self.id == other.id
	}
}

impl<'a> cmp::PartialOrd for Title<'a> {
	fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl<'a> cmp::Eq for Title<'a> {}

impl<'a> cmp::Ord for Title<'a> {
	fn cmp(&self, other: &Self) -> cmp::Ordering {
		self.id.cmp(&other.id)
	}
}

use std::hash;

impl<'a> hash::Hash for Title<'a> {
	fn hash<H: hash::Hasher>(&self, state: &mut H) {
		self.id.hash(state);
	}
}

#[derive(Debug, Clone, Deserialize)]
pub struct JashinJson {
	reference_date: DateTime<Local>,
	area_names: Vec<String>,
	announcement: String,
	announcement_at_start: String,
	announcement_at_end: String,
	information: String,

	#[serde(deserialize_with = "transform_string_to_regex")]
	nickname_regex: regex::Regex,
	tables: Vec<TableJson>,
}

#[derive(Debug, Clone, Deserialize)]
struct TableJson {
	start_day: u32,
	titles: Vec<TitleJson>,
}

#[derive(Debug, Clone, Deserialize)]
struct TitleJson {
	id: String,
	display: String,
	monster_ids: Vec<String>,
}

#[cfg(test)]
pub(crate) mod tests {
	use super::*;
	use chrono::offset::TimeZone;

	#[test]
	fn test_title() {
		let jashin = data();

		// 1st title
		assert_eq!(
			jashin.title(Local.ymd(2020, 7, 10).and_hms(6, 0, 0)).id,
			"five_elemental_armors"
		);

		// Edge of 1st title
		assert_eq!(
			jashin.title(Local.ymd(2020, 7, 25).and_hms(5, 59, 59)).id,
			"five_elemental_armors"
		);

		// 2nd title
		assert_eq!(
			jashin.title(Local.ymd(2020, 7, 25).and_hms(6, 0, 0)).id,
			"dream_masako"
		);

		// Edge of 2nd title
		assert_eq!(
			jashin.title(Local.ymd(2020, 8, 10).and_hms(5, 59, 59)).id,
			"dream_masako"
		);


		// 3rd title
		assert_eq!(
			jashin.title(Local.ymd(2020, 8, 10).and_hms(6, 0, 0)).id,
			"malucia_thoma"
		);

		// 4th title
		assert_eq!(
			jashin.title(Local.ymd(2020, 8, 25).and_hms(6, 0, 0)).id,
			"malucia_masako"
		);

		// last title of 1st period of 1st table
		assert_eq!(
			jashin.title(Local.ymd(2020, 11, 10).and_hms(6, 0, 0)).id,
			"raz_zel"
		);

		// 1st title of 2nd period of 1st table
		assert_eq!(
			jashin.title(Local.ymd(2020, 12, 10).and_hms(6, 0, 0)).id,
			"five_elemental_armors"
		);

		// 6th title of 1st period of 2nd table
		assert_eq!(
			jashin.title(Local.ymd(2020, 12, 25).and_hms(6, 0, 0)).id,
			"zel_masako"
		);

		// Last title of 1st period of 2nd table
		assert_eq!(
			jashin.title(Local.ymd(2021, 4, 25).and_hms(6, 0, 0)).id,
			"calamity_malucia"
		);

		// 1st title of 2nd period of 2nd table
		assert_eq!(
			jashin.title(Local.ymd(2021, 5, 25).and_hms(6, 0, 0)).id,
			"dream_masako"
		);
	}

	#[test]
	fn test_past_date() {
		let jashin = data();

		// Edge of title before reference date
		assert_eq!(
			jashin.title(Local.ymd(2020, 7, 10).and_hms(5, 59, 59)).id,
			"calamity_malucia"
		);
		assert_eq!(
			jashin.title(Local.ymd(2020, 6, 25).and_hms(6, 0, 0)).id,
			"calamity_malucia"
		);

		// 2 titles before the reference date
		assert_eq!(
			jashin.title(Local.ymd(2020, 6, 10).and_hms(6, 0, 0)).id,
			"raz_zel"
		);
	}

	pub(crate) fn data<'a>() -> Jashin<'a> {
		let mut inner: JashinJson = serde_json::from_str(TEST_DATA).unwrap();

		inner.tables.sort_by(|a, b| a.start_day.cmp(&b.start_day));

		Jashin {
			tables: Tables::new(&inner.tables, inner.reference_date).unwrap(),
			inner,
		}
	}

	const TEST_DATA: &str = r#"
        {
			"reference_date": "2020-07-10T06:00:00.000+09:00",
        	"area_names": ["一獄", "二獄", "三獄", "四獄", "五獄", "はい？獄"],
        	"announcement": "本日の邪神の宮殿は __TITLE__ です！",
        	"announcement_at_start": "邪神の宮殿は本日から __TITLE__ です！相手は __MONSTERS__、あると良い耐性は __RESISTANCES__ です！",
			"announcement_at_end": "本日の邪神の宮殿は __TITLE1__ です！明日からは __TITLE2__ が始まります！",
        	"information": "本日の邪神の宮殿は __TITLE__ です！相手は__MONSTERS__、あると良い耐性は __RESISTANCES__ です！",
			"nickname_regex": "(?:邪神|じゃしん|ジャシン|ｼﾞｬｼﾝ)",
        	"tables": [
        		{
        			"start_day": 10,
        			"titles": [
                		{
                			"id": "five_elemental_armors",
                			"display": "五属性の災禍",
                			"monster_ids": ["jashin_armors"]
                		},
                		{
                			"id": "malucia_thoma",
                			"display": "闇に堕ちた英雄の幻影",
                			"monster_ids": ["jashin_malucia", "jashin_thoma"]
                		},
                		{
                			"id": "calamity_dream",
                			"display": "災いの神話と暴虐の悪夢",
                			"monster_ids": ["jashin_calamity", "jashin_dream"]
                		},
                		{
                			"id": "masako_pictures",
                			"display": "魔幻の覇王軍",
                			"monster_ids": ["jashin_masako", "jashin_pictures"]
                		},
                		{
                			"id": "raz_zel",
                			"display": "覇道の双璧",
                			"monster_ids": ["jashin_raz", "jashin_zel"]
                		}
        			]
        		},
        		{
        			"start_day": 25,
        			"titles": [
                		{
                			"id": "dream_masako",
                			"display": "破壊と創造の神々",
                			"monster_ids": ["jashin_dream", "jashin_masako"]
                		},
                		{
                			"id": "malucia_masako",
                			"display": "背離する魔族の血統",
                			"monster_ids": ["jashin_malucia", "jashin_masako"]
                		},
                		{
                			"id": "raz_pictures",
                			"display": "魔宮の守護者たち",
                			"monster_ids": ["jashin_raz", "jashin_pictures"]
                		},
                		{
                			"id": "dream_thoma",
                			"display": "昏き悪夢の衝撃",
                			"monster_ids": ["jashin_dream", "jashin_thoma"]
                		},
                		{
                			"id": "calamity_pictures",
                			"display": "災厄神話ギャラリー",
                			"monster_ids": ["jashin_calamity", "jashin_pictures"]
                		},
                		{
                			"id": "zel_masako",
                			"display": "覇業の君臣",
                			"monster_ids": ["jashin_zel", "jashin_masako"]
                		},
                		{	"id": "calamity_thoma",
                			"display": "悲劇の英雄譚",
                			"monster_ids": ["jashin_calamity", "jashin_thoma"]
                		},
                		{
                			"id": "dream_malucia",
                			"display": "見果てぬ夢の蛮勇",
                			"monster_ids": ["jashin_dream", "jashin_malucia"]
                		},
                		{
                			"id": "thoma_zel",
                			"display": "魔幻の最高幹部",
                			"monster_ids": ["jashin_thoma", "jashin_zel"]
                		},
                		{
                			"id": "calamity_malucia",
                			"display": "妖女と災獣",
                			"monster_ids": ["jashin_calamity", "jashin_malucia"]
                		}
        			]
        		}
        	]
		}
	"#;
}
