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
	utils::{
		transform_string_to_regex,
		transform_vec_string_to_vec_regex,
	},
};
use super::{
	Responder,
	ResponseCriteria,
	rate_limit::RateLimit,
};

const DATA: &str = "drakeema-data/response_status.json";

pub struct StatusResponse<'a> {
	conn: &'a Connection,
	emojis: Emojis<'a>, 
	responders: Vec<&'a dyn Responder>,
	keema: &'a contents::Keema,
	config: Config,
	rate_limit: RateLimit,
}

impl<'a> StatusResponse<'a> {
	pub fn load(
		conn: &'a Connection,
		responders: Vec<&'a dyn Responder>,
		keema: &'a contents::Keema
	) -> Result<Self> {
		let config: Config = serde_json::from_reader(
			BufReader::new(File::open(DATA)?)
		)
		.map_err(|e| Error::ParseJsonError(DATA.to_owned(), e))?;

		let emojis = Emojis::load(&conn)?;
		let rate_limit = RateLimit::new(config.rate_limit);
		Ok(StatusResponse {
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

		if self.is_ignore(status.account().acct()) {
			info!("Ignore status: acct: {}", status.account().acct());
			return Ok(());
		}

        let response: Option<String>;

        if self.is_oshiete_keemasan(&content) {
            info!("Match Keywords for OSHIETE: {}", content);

			let rc = ResponseCriteria::new(chrono::Local::now(), content);
            let mut r = self.responders.iter()
                .map(|i| i.respond(&rc))
                .filter(|i| i.is_some())
                .map(|i| i.unwrap())
                .collect::<Vec<String>>()
                .join("\n");

            if r.is_empty() {
                r = String::from("？");
            }

			info!("StatusResponse for OSHIETE: {}", r);
			response = Some(r);
		} else if self.is_keemasan(&content) && self.is_healthcheck(content){
			info!("Match keywords for healthcheck: {}", content);

			use chrono::{
				Timelike,
				Local,
			};

			response = self.config.healthcheck_responses.get(
				Local::now().second() as usize % self.config.healthcheck_responses.len()
			).map(|r| r.to_owned());
        } else {
            response = self.keema.respond(&ResponseCriteria::new(chrono::Local::now(), content));
        }
        trace!("StatusResponse created: {:?}", response);

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
                Ok(_) => info!("StatusResponse completed"),
                Err(e) => error!("Failed to send reply to {}: {}", status.account().acct(), e),
            };
		}
		
		Ok(())
	}

	fn is_ignore(&self, acct: &str) -> bool {
		self.config.ignore_acct_regex.iter().any(|re| re.is_match(acct))
	}

	fn is_healthcheck(&self, text: &str) -> bool {
		self.config.healthcheck_regex.is_match(text)
	}

	fn is_oshiete(&self, text: &str) -> bool {
		self.config.oshiete_regex.is_match(text)
	}

	fn is_keemasan(&self, text: &str) -> bool {
		self.config.keemasan_regex.is_match(text) 
	}

	fn is_oshiete_keemasan(&self, text: &str) -> bool {
		self.is_oshiete(text) && self.is_keemasan(text)
	}
}

#[derive(Debug, Clone, Deserialize)]
struct Config {
    #[serde(deserialize_with = "transform_string_to_regex")]
    keemasan_regex: regex::Regex,
    #[serde(deserialize_with = "transform_string_to_regex")]
	oshiete_regex: regex::Regex,
	#[serde(deserialize_with = "transform_string_to_regex")]
	healthcheck_regex: regex::Regex,
	
	rate_limit: usize,
	healthcheck_responses: Vec<String>,

	#[serde(deserialize_with = "transform_vec_string_to_vec_regex")]
	ignore_acct_regex: Vec<regex::Regex>,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_is_ignore() {
		let conn = Connection::new().unwrap();
		let keema = contents::keema::tests::data();
		let resp = data(&conn, vec![], &keema);

		assert!(resp.is_ignore("hoge@example.com"));
		assert!(resp.is_ignore("hoge@fuga.com"));
		assert!(resp.is_ignore("hoge@fuga.pya.hoge.foresdon.jp"));
		assert!(!resp.is_ignore("kedama@foresdon.jp"));
	}

	fn data<'a>(conn: &'a Connection, responders: Vec<&'a dyn Responder>, keema: &'a contents::Keema) -> StatusResponse<'a> {
		let config = serde_json::from_str::<Config>(DATA).unwrap();
		StatusResponse {
			conn,
			emojis: crate::emojis::tests::data(conn),
			responders,
			keema,
			rate_limit: RateLimit::new(config.rate_limit),
			config,
		}
	}

	const DATA: &str = r#"
		{
			"keemasan_regex": "キーマさん",
			"oshiete_regex": "(?:(?:おし|教)えて|(?:てぃーち|ティーチ|ﾃｨーﾁ)\\s*(?:みー|ミー|ﾐー))",
			"rate_limit": 20,
			"healthcheck_regex": "(?:(?:元気|げんき|ゲンキ|ｹﾞﾝｷ)(?:!|！)*(?:？|\\?))",
			"healthcheck_responses": [
				"元気です！",
				"元気な場合がある。",
				":m_drakeema:",
				":x_exkun:",
				":m_drakee: :m_drakeema: :m_drakeemage: :m_drakeetaho:",
				":d_sleep:"
			],
			"ignore_acct_regex": [
				"@example.com$",
				"^hoge@fuga.com$",
				"hoge.foresdon.jp$"
			]
		}
	"#;
}
