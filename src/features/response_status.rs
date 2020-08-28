use std::fs::File;
use std::io::BufReader;
use mastors::{
	Connection,
	api::v1::statuses,
	entities::Status,
};
use serde::Deserialize;
use crate::{
	Emojis,
	Error,
	Result,
	contents,
	utils::transform_string_to_regex,
};
use super::{
	Responder,
	ResponseCriteria,
	rate_limit::RateLimit,
};

const DATA: &str = "drakeema-data/response_status.json";

pub struct Response<'a> {
	conn: &'a Connection,
	emojis: Emojis<'a>, 
	responders: Vec<&'a dyn Responder>,
	keema: &'a contents::Keema,
	config: OshieteConfig,
	rate_limit: RateLimit,
}

impl<'a> Response<'a> {
	pub fn load(
		conn: &'a Connection,
		responders: Vec<&'a dyn Responder>,
		keema: &'a contents::Keema
	) -> Result<Self> {
		let config: OshieteConfig = serde_json::from_reader(
			BufReader::new(File::open(DATA)?)
		)
		.map_err(|e| Error::ParseJsonError(DATA.to_owned(), e))?;

		let emojis = Emojis::load(&conn)?;
		let rate_limit = RateLimit::new(config.rate_limit);
		Ok(Response {
			conn,
			emojis,
			responders,
			keema,
			config,
			rate_limit,
		})
	}

	pub fn process(&mut self, status: &Status) -> Result<()> {
        let content = match status.content() {
            Some(content) => content,
            None => return Ok(()),
        };
        trace!("Status received: {:?}", status);

        let response: Option<String>;

        if self.config.keemasan_regex.is_match(content) && self.config.oshiete_regex.is_match(content) {
            trace!("Match Keywords for OSHIETE: {}", content);

            let mut r = self.responders.iter()
                .map(|i| i.respond(&ResponseCriteria::new(chrono::Local::now(), content)))
                .filter(|i| i.is_some())
                .map(|i| i.unwrap())
                .collect::<Vec<String>>()
                .join("\n");

            if r.is_empty() {
                r = String::from("？");
            }

            response = Some(r);
        } else {
            trace!("Do not match Keywords for OSHIETE: {}", content);
            response = self.keema.respond(&ResponseCriteria::new(chrono::Local::now(), content));
        }
        trace!("Reaction created: {:?}", response);

        if let Some(response) = response {
			if let Err(e) = self.rate_limit.increment() {
				error!("Respond rate limit exceeded: {}", e);
				return Err(e);
			}

			info!("Reply to {}: status: {}", status.account().acct(), &response);
            let response = self.emojis.emojify(
                String::from("@") + status.account().acct() + "\n\n" + response.as_str()
            );
    
            let mut post = statuses::post(self.conn, response);
            if status.account().is_remote() {
                post = post.in_reply_to_id(status.id());
            }
    
            match post.send() {
                Ok(posted) => info!(
                    "Response completed: status: {:?}: mention: {}",
                    posted.status().unwrap().content(),
                    status.account().acct(),
                ),
                Err(e) => error!("Can't send reply to {}: {}", status.account().acct(), e),
            };
		}
		
		Ok(())
	}

}

#[derive(Debug, Clone, Deserialize)]
struct OshieteConfig {
    #[serde(deserialize_with = "transform_string_to_regex")]
    keemasan_regex: regex::Regex,
    #[serde(deserialize_with = "transform_string_to_regex")]
	oshiete_regex: regex::Regex,
	
	rate_limit: usize,
}

