use std::collections::HashMap;
use std::fs::{ File, self };
use std::io::BufReader;
use regex::Regex;
use serde::Deserialize;
use crate::{
	Error,
	Result,
	resistances::{ Resistance, Resistances },
	utils::transform_string_to_regex,
};

const DATA_DIR: &str = "data/monsters";

pub struct Monsters {
	inner: HashMap<String, Monster>,
}

impl Monsters {
	pub fn load() -> Result<Self> {
		let mut inner = HashMap::new();

		let files = fs::read_dir(DATA_DIR)?
			.filter_map(|dir_entry| {
    			let dir_entry = dir_entry.ok()?;
    			if dir_entry.file_type().ok()?.is_file() &&
    				dir_entry.path().extension()? == "json" {
    					Some(dir_entry.path())
    			} else {
    				None
    			}
    		});
    	
    	for file in files {
    		let m: Monster = serde_json::from_reader(
    			BufReader::new(File::open(&file)?)
			)
			.map_err(|e| Error::ParseJsonError(file.to_string_lossy().to_string(), e))?;    

    		inner.insert(m.id().to_owned(), m);
    	}
    
    	Ok(Monsters {
			inner,
		})
	}
}

impl std::ops::Deref for Monsters {
	type Target = HashMap<String, Monster>;
	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

#[derive(Debug, Clone, Deserialize)]
pub struct Monster {
	id: String,
	display: String,
	official_name: String,
	#[serde(deserialize_with="transform_string_to_regex")]
	nickname_regex: Regex,
	resistances: Resistances<Vec<Vec<Resistance>>>,
}

impl Monster {
	/// Get an ID of this monster.
	pub fn id(&self) -> &str {
		self.id.as_ref()
	}

	/// Get the common name of the monster displayed in the status.
	pub fn display(&self) -> &str {
		self.display.as_ref()
	}

	/// Get the official name of the monster.
	pub fn official_name(&self) -> &str {
		self.official_name.as_ref()
	}

	/// Get the `Regex` for determining which monster.
	pub fn nickname_regex(&self) -> &Regex {
		&self.nickname_regex
	}

	/// Get the resistances to need to battle the monster.
	pub fn resistances(&self) -> &Resistances<Vec<Vec<Resistance>>> {
		&self.resistances
	}

	pub fn is_match(&self, text: impl AsRef<str>) -> bool {
		self.nickname_regex().is_match(text.as_ref())
	}
}
