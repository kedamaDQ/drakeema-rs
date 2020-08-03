use std::fs::File;
use std::io::BufReader;
use chrono::{ Datelike, DateTime, Duration, Local };
use serde::Deserialize;
use crate::{
	Error,
	monsters::Monsters,
	resistances::Resistances,
	Result,
};

const DATA: &str = "data/contents/jashin.json";

#[derive(Debug, Clone)]
pub struct Jashin<'a, T> {
	monsters: &'a Monsters,
	inner: Box<T>,
}

impl<'a> Jashin<'a, JashinJson> {
	pub fn load(monsters: &'a Monsters) -> Result<Self> {
    	let mut inner: JashinJson = match serde_json::from_reader(
    		BufReader::new(File::open(DATA)?)
    	) {
    		Ok(jj) => jj,
    		Err(e) => return Err(
    			Error::ParseJsonError(DATA.to_owned(), e)
    		),
		};

		inner.tables.sort_by(|a, b| a.start_day.cmp(&b.start_day));

		Ok(Jashin {
			monsters,
			inner: Box::new(inner),
		})
	}

	pub fn announcement(&self, at: DateTime<Local>) -> String {
		use std::ops::Add;

		let title_today = self.title(at);
		let title_tomorrow = self.title(at.add(Duration::hours(24)));
		let title_yesterday = self.title(at.add(Duration::hours(-24)));

		if title_today != title_yesterday {
			// Date is start date of the period
			self.announcement_at_start
				.replace("__TITLE__", title_today.display_title())
				.replace("__MONSTERS__", title_today.display_monsters(self.monsters).as_str())
				.replace("__RESISTANCES__", title_today.display_resistances(self.monsters, Some(&self.area_names)).as_str())
		} else if title_today != title_tomorrow {
			// Date is end date of period
			self.announcement_at_end
				.replace("__TITLE1__", title_today.display_title())
				.replace("__TITLE2__", title_tomorrow.display_title())
		} else {
			// Date is duaring the period
			self.announcement
				.replace("__TITLE__", title_today.display_title())
		}
	}

	pub fn information(&self, at: DateTime<Local>) -> String {
		let title = self.title(at);
		self.information
			.replace("__TITLE__", title.display_title())
			.replace(
				"__MONSTERS__",
				title.display_monsters(&self.monsters).as_str()
			)
			.replace(
				"__RESISTANCES__",
				title.display_resistances(&self.monsters, Some(&self.area_names)).as_str()
			)
	}

	fn tables(&self) -> &Vec<Table> {
		&self.tables
	}

	fn title(&self, at: DateTime<Local>) -> &Title {
		let titles = &self.table(at).titles;
		if titles.is_empty() {
			panic!("Jashin titles is empty");
		}

		let elapsed_months = self.elapsed_months(at);
		let elapsed_months = if elapsed_months < 0i32 {
			let len = titles.len() as i32;
			(len + elapsed_months % len).abs() as usize
		} else {
			elapsed_months as usize % titles.len()
		};

		titles.get(elapsed_months).unwrap()
	}

	fn table(&self, at: DateTime<Local>) -> &Table {
		self.tables().iter()
			.rev()
			.find(|table| {
				at.day() > table.start_day ||
				(at.day() == table.start_day && at.time() >= self.reference_date.time())
			})
			.unwrap_or(
				&self.tables().last().expect("No Jashin table found")
			)
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

impl<'a, T> std::ops::Deref for Jashin<'a, T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.inner
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
	tables: Vec<Table>,
}

#[derive(Debug, Clone, Deserialize)]
struct Table {
	start_day: u32,
	titles: Vec<Title>,
}

#[derive(Debug, Clone, Deserialize)]
struct Title {
	id: String,
	display: String,
	monster_ids: Vec<String>,
}

impl Title {
	fn display_title(&self) -> &str {
		self.display.as_str()
	}

	fn display_monsters(&self, monsters: &Monsters) -> String {
		self.monster_ids.iter()
			.map(|id| monsters
				.get(id)
				.expect("Unknown monster ID")
				.display()
				.to_owned()
			)
			.collect::<Vec<String>>()
			.join("と")
	}

	fn display_resistances<T, U>(&self, monsters: &Monsters, area_names: Option<T>) -> String
	where
		T: AsRef<[U]>,
		U: AsRef<str>,
	{
		self.monster_ids.iter()
			.map(|id| monsters
				.get(id)
				.expect("Unknown monster ID")
				.resistances()
			)
			.fold(Resistances::new(), |acc, r| acc.join(r))
			.display(area_names)
	}
}

use std::cmp;

impl cmp::PartialEq for Title {
	fn eq(&self, other: &Self) -> bool {
		self.id == other.id
	}
}

impl cmp::PartialOrd for Title {
	fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl cmp::Eq for Title {}

impl cmp::Ord for Title {
	fn cmp(&self, other: &Self) -> cmp::Ordering {
		self.id.cmp(&other.id)
	}
}

use std::hash;

impl hash::Hash for Title {
	fn hash<H: hash::Hasher>(&self, state: &mut H) {
		self.id.hash(state);
	}
}
#[cfg(test)]
mod tests {
	use super::*;
	use chrono::offset::TimeZone;

	#[test]
	fn test_title() {
		let monsters = crate::monsters::load().unwrap();
		let jashin = data(&monsters);

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
		let monsters = crate::monsters::load().unwrap();
		let jashin = data(&monsters);

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

	fn data(monsters: &Monsters) -> Jashin<JashinJson> {
		let inner = serde_json::from_str(TEST_DATA).unwrap();
		Jashin {
			monsters,
			inner: Box::new(inner),
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
