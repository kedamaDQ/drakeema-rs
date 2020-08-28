use std::fmt;
use std::str::FromStr;
use regex::Regex;
use serde::{
	de::{
		Deserialize,
		Error,
		SeqAccess,
		Visitor,
		self,
	},
};

pub fn transform_string_to_regex<'de, D>(deserializer: D) -> std::result::Result<Regex, D::Error>
where
	D: de::Deserializer<'de>,
{
	let s = String::deserialize(deserializer)?;
	Regex::from_str(&s).map_err(D::Error::custom)
}

pub fn transform_vec_string_to_vec_regex<'de, D>(deserializer: D) -> std::result::Result<Vec<Regex>, D::Error>
where
	D: de::Deserializer<'de>,
{

	struct RegexVisitor;

	impl<'de> Visitor<'de> for RegexVisitor {
		type Value = Vec<Regex>;

		fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
			formatter.write_str("a string")
		}

		fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
		where
			A: SeqAccess<'de>,
		{
			let mut vec = Vec::new();
			while let Some(s) = seq.next_element()? {
				vec.push(Regex::from_str(s).map_err(A::Error::custom)?);
			};
			Ok(vec)
		}
	}

	deserializer.deserialize_seq(RegexVisitor)
}
