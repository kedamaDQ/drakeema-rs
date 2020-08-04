use err_derive::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
	#[error(display = "Parse json error: {}, {}", _0, _1)]
	ParseJsonError(
		String,
		#[error(source)]
		serde_json::Error,
	),

	#[error(display = "IO error: {}", _0)]
	IoError(
		#[error(souce, from)]
		std::io::Error,
	),

	#[error(display = "Data not presented: {}", _0)]
	DataNotPresentError(
		String,
	),

	#[error(display = "Unknown monster id: {}", _0)]
	UnknownMonsterIdError(
		String,
	),

	#[error(display = "Invalid regex: {}", _0)]
	InvalidRegexError(
		#[error(source, from)]
		regex::Error,
	),
}
