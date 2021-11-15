use err_derive::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
	#[error(display = "parse json error: {}, {}", _0, _1)]
	UnparseableJson(
		String,
		#[error(source)]
		serde_json::Error,
	),

	#[error(display = "IO error: {}", _0)]
	Io(
		#[error(souce, from)]
		std::io::Error,
	),

	#[error(display = "Data not presented: file: {}, data: {}", _0, _1)]
	DataNotPresented(
		&'static str,
		String,
	),

	#[error(display = "Unknown monster ID: file: {}, id: {}", _0, _1)]
	UnknownMonsterId(
		&'static str,
		String,
	),

	#[error(display = "Invalid regex: {}", _0)]
	InvalidRegex(
		#[error(source, from)]
		regex::Error,
	),

	#[error(display = "Mastors API call error: {}", _0)]
	MastorsApi(
		#[error(source, from)]
		mastors::Error,
	),

	#[error(display = "Lost the streaming connection: timeline: {}, retry: {}", _0, _1)]
	LostStreamingConnection(
		mastors::streaming::StreamType,
		usize,
	),

	#[error(display = "Failed to load tmporary data: {}, {}", _0, _1)]
	LoadTmpData(
		String,
		std::io::Error,
	),

	#[error(display = "Unexpected temporary data format: {}, {}", _0, _1)]
	TmpDataFormat(
		String,
		std::num::ParseIntError,
	),

	#[error(display = "Failed to save temporary data: {}, {}", _0, _1)]
	SaveTmpData(
		String,
		std::io::Error,
	),

	#[error(display = "Rate limit exceeded: limit: {}", _0)]
	ExceedRateLimit(
		usize
	),

	#[error(display = "HTTP request error: {}", _0)]
	HttpRequest(
		#[error(source, from)]
		reqwest::Error,
	),

	#[error(display = "Failed to parse feed: {}, {}", _0, _1)]
	UnparseableFeed(
		String,
		feed_rs::parser::ParseFeedError,
	),
}
