use std::str::FromStr;
use regex::Regex;
use serde::{ de, de::Error, de::Deserialize };

pub fn transform_string_to_regex<'de, D>(deserializer: D) -> std::result::Result<Regex, D::Error>
where
	D: de::Deserializer<'de>,
{
	let s = String::deserialize(deserializer)?;
	Regex::from_str(&s).map_err(D::Error::custom)
}
