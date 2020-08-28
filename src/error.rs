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

	#[error(display = "Mastors API call error: {}", _0)]
	MastorsApiError(
		#[error(source, from)]
		mastors::Error,
	),

	#[error(display = "Failed to send Status to channel: {}", _0)]
	SendMessageError(
		#[error(source, from)]
		Box<std::sync::mpsc::SendError<crate::listeners::Message>>,
	),

	#[error(display = "Lost the streaming connection: timeline: {}, retry: {}", _0, _1)]
	LostStreamingConnectionError(
		mastors::streaming::StreamType,
		usize,
	),

	#[error(display = "Failed to load tmporary data: {}, {}", _0, _1)]
	LoadTmpDataError(
		String,
		std::io::Error,
	),

	#[error(display = "Unexpected temporary data format: {}, {}", _0, _1)]
	TmpDataFormatError(
		String,
		std::num::ParseIntError,
	),

	#[error(display = "Failed to save temporary data: {}, {}", _0, _1)]
	SaveTmpDataError(
		String,
		std::io::Error,
	),
}
