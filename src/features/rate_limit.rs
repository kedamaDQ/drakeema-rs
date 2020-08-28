use chrono::{ DateTime, Duration, Local };
use crate::{
	Error,
	Result,
};

const RESET_INTERVAL: i64 = 1;

pub struct RateLimit {
	from: DateTime<Local>,
	limit: usize,
	count: usize,
}

impl RateLimit {
	pub fn new(limit: usize) -> Self {
		RateLimit {
			from: Local::now(),
			limit,
			count: 0,
		}
	}

	pub fn increment(&mut self) -> Result<usize> {
		trace!("from: {}, now: {}, count: {}", self.from, Local::now(), self.count);

		let count = if Local::now() - self.from > Duration::minutes(RESET_INTERVAL) {
			self.from = Local::now();
			self.count = 1;
			self.count
		} else {
			self.count += 1;

			if self.count > self.limit {
				return Err(Error::ExceedRateLimitError(self.limit))
			} else {
				self.count
			}
		};

		info!("Rate limit status: from: {}, count: {}", self.from, self.count);
		Ok(count)
	}
}
