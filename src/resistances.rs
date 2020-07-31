use serde::de;

/// Represent resistances of abnormal conditions.
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum Resistance {
	Spell,		// 呪文
	Breath,		// ブレス
	Sleep,		// 眠り
	Confusion,	// 混乱
	Paralisys,	// マヒ
	Death,		// 即死
	Seal,		// 封印
	Illusion,	// 幻惑
	Dance,		// 踊り
	Poison,		// どく
	Charm,		// 魅了
	Curse,		// 呪い
	Fall,		// 転び
	Bind,		// しばり
	Fear,		// おびえ
	Laugh,		// 笑い
	Flame,		// 炎
	Ice,		// 氷
	Breeze,		// 風
	Thunder,	// 雷
	Earth,		// 土
	Light,		// 光
	Dark,		// 闇
	Various(String),	// 不定
}

impl std::fmt::Display for Resistance {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
        	Resistance::Spell => write!(f, "呪文"),
        	Resistance::Breath => write!(f, "ブレス"),
        	Resistance::Sleep => write!(f, "眠り"),
        	Resistance::Confusion => write!(f, "混乱"),
        	Resistance::Paralisys => write!(f, "マヒ"),
			Resistance::Death => write!(f, "即死"),
        	Resistance::Seal => write!(f, "封印"),
			Resistance::Illusion => write!(f, "幻惑"),
        	Resistance::Dance => write!(f, "踊り"),
			Resistance::Poison => write!(f, "どく"),
        	Resistance::Charm => write!(f, "魅了"),
        	Resistance::Curse => write!(f, "呪い"),
        	Resistance::Fall => write!(f, "転び"),
        	Resistance::Bind => write!(f, "しばり"),
        	Resistance::Fear => write!(f, "おびえ"),
        	Resistance::Laugh => write!(f, "笑い"),
        	Resistance::Flame => write!(f, "炎"),
			Resistance::Ice => write!(f, "氷"),
			Resistance::Breeze => write!(f, "風"),
        	Resistance::Thunder => write!(f, "雷"),
        	Resistance::Earth => write!(f, "土"),
        	Resistance::Light => write!(f, "光"),
        	Resistance::Dark => write!(f, "闇"),
			Resistance::Various(s) => write!(f, "{}", s),
		}
	}
}

use std::str::FromStr;

impl FromStr for Resistance {
	type Err = crate::Error;

	fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
		match s {
        	"呪文" => Ok(Resistance::Spell),
        	"ブレス" => Ok(Resistance::Breath),
        	"眠り" => Ok(Resistance::Sleep),
        	"混乱" => Ok(Resistance::Confusion),
        	"マヒ" => Ok(Resistance::Paralisys),
			"即死" => Ok(Resistance::Death),
        	"封印" => Ok(Resistance::Seal),
			"幻惑" => Ok(Resistance::Illusion),
        	"おびえ" => Ok(Resistance::Fear),
			"どく" => Ok(Resistance::Poison),
        	"魅了" => Ok(Resistance::Charm),
        	"呪い" => Ok(Resistance::Curse),
        	"転び" => Ok(Resistance::Fall),
        	"しばり" => Ok(Resistance::Bind),
        	"踊り" => Ok(Resistance::Dance),
        	"笑い" => Ok(Resistance::Laugh),
        	"炎" => Ok(Resistance::Flame),
			"氷" => Ok(Resistance::Ice),
			"風" => Ok(Resistance::Breeze),
        	"雷" => Ok(Resistance::Thunder),
        	"土" => Ok(Resistance::Earth),
        	"闇" => Ok(Resistance::Dark),
        	"光" => Ok(Resistance::Light),
			_ => Ok(Resistance::Various(s.to_owned())),
		}
	}
}

impl<'de> de::Deserialize<'de> for Resistance {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
		let s = String::deserialize(deserializer)?;
		match Resistance::from_str(s.as_str()) {
			Ok(r) => Ok(r),
			Err(e) => Err(de::Error::custom(e)),
		}
    }
}

#[derive(Debug, Clone)]
pub struct Resistances<T> {
	inner: Box<T>,
}

impl<T> std::ops::Deref for Resistances<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl Resistances<Vec<Vec<Resistance>>> {
	pub fn new() -> Self {
		Resistances {
			inner: Box::new(vec![vec![]]),
		}
	}

	pub fn join(&self, other: &Self) -> Self {
		use std::collections::HashSet;

		if self.len() != other.len() && self.len() != 1 && other.len() != 1 {
			panic!("Cannot join Resistances: self.len: {}, other.len: {}", self.len(), other.len());
		}

		let other = other.clone();

    	let larger: &Vec<Vec<Resistance>>;
    	let smaller: &Vec<Vec<Resistance>>;
    	if self.len() > other.len() {
    		larger =  &self;
    		smaller = &other;
    	} else {
			larger = &other;
    		smaller = &self;
		}

		Resistances {
			inner: Box::new(
				larger.iter()
    			.zip(smaller.iter().cycle())
    			.map(|(l, s)| {
    				let mut res = l.iter()
    					.chain(s.iter())
    					.map(|resistance| resistance.to_owned())
    					.collect::<HashSet<Resistance>>()
    					.iter()
    					.map(|resistance| resistance.to_owned())
						.collect::<Vec<Resistance>>();
					res.sort();
					res
    			})
				.collect::<Vec<Vec<Resistance>>>()
			),
		}
	}

	pub fn display<T, U>(&self, area_names: Option<T>) -> String
	where
		T: AsRef<[U]>,
		U: AsRef<str>,
	{
		if self.len() == 1 {
			self.iter()
				.next()
				.expect("Resistances is not set")
				.iter()
				.map(|r| r.to_string())
				.collect::<Vec<String>>()
				.join("、")
		} else {
			let area_names = area_names.expect("Area names not found for multiple resistance");
			self.iter()
				.enumerate()
				.map(|(i, resistances)| {
					area_names.as_ref()[i].as_ref().to_string() + "は " +
					resistances.iter()
						.map(|r| r.to_string())
						.collect::<Vec<String>>()
						.join("、")
						.as_ref()
				})
				.collect::<Vec<String>>()
				.join("、")
		}
	}


}

impl<'de> de::Deserialize<'de> for Resistances<Vec<Vec<Resistance>>> {
	fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
	where
		D: de::Deserializer<'de>,
	{
		let v1 = Vec::deserialize(deserializer)?;
		Ok(Resistances {
			inner: Box::new(v1),
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_order() {
		assert!(Resistance::Spell < Resistance::Breath);
	}

	#[test]
	fn test_sort() {
		let mut vec = vec![Resistance::Breath, Resistance::Spell];
		vec.sort();
		assert_eq!(vec, vec![Resistance::Spell, Resistance::Breath]);
	}
}
