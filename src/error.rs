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

	#[error(display = "Data not presented: file: {}, data: {}", _0, _1)]
	DataNotPresentError(
		&'static str,
		String,
	),

	#[error(display = "Unknown monster ID: file: {}, id: {}", _0, _1)]
	UnknownMonsterIdError(
		&'static str,
		String,
	),

	#[error(display = "Invalid regex: {}", _0)]
	InvalidRegexError(
		#[error(source, from)]
		regex::Error,
	),
}
