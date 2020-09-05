use std::fs::File;
use std::io::BufReader;
use std::sync::mpsc;
use mastors::entities::Status;
use serde::Deserialize;
use crate::{
	Error,
	Message,
	Monsters,
	Result,
	contents::*,
	utils::{
		transform_string_to_regex,
		transform_vec_string_to_vec_regex,
	},
};

const DATA: &str = "drakeema-data/features/response/status.json";

pub struct StatusProcessor {
	responders: Vec<Box<dyn Responder>>,
	keema: Keema,
	config: Config,
}

impl StatusProcessor {
	pub fn load() -> Result<Self> {
		info!("Initialize StatusProcessor");

		let config: Config = serde_json::from_reader(
			BufReader::new(File::open(DATA)?)
		)
		.map_err(|e| Error::ParseJsonError(DATA.to_owned(), e))?;

		let responders: Vec<Box<dyn Responder>> = vec![
			Box::new(Jashin::load()?),
			Box::new(Seishugosha::load()?),
			Box::new(Boueigun::load()?),
			Box::new(Monsters::load()?),
		];

		let keema = Keema::load()?;

		Ok(StatusProcessor {
			responders,
			keema,
			config,
		})
	}

	pub fn process(&self, tx: &mpsc::Sender<Message>, status: &Status) {
        let content = match status.content() {
            Some(content) => content,
            None => return,
        };
        trace!("Status received: {:?}", status);

		if self.is_ignore(status.account().acct()) {
			info!("Ignore status: acct: {}", status.account().acct());
			return;
		}

        let response: Option<String>;

        if self.is_oshiete_keemasan(&content) {
            info!("Text matched keywords of Oshiete: {}", content);

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

			response = Some(r);
		} else if self.is_keemasan(&content) && self.is_healthcheck(content){
			info!("Text matched keywords of healthcheck: {}", content);

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

        if let Some(response) = response {
			tx.send(Message::Status {
				text: response,
				mention: Some(status.account().acct().to_owned()),
				in_reply_to_id: if status.account().is_remote() {
					Some(status.id().to_owned())
				} else {
					None
				}
			}).unwrap();
		}
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
	
	healthcheck_responses: Vec<String>,

	#[serde(deserialize_with = "transform_vec_string_to_vec_regex")]
	ignore_acct_regex: Vec<regex::Regex>,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_is_ignore() {
		let resp = data(vec![]);

		assert!(resp.is_ignore("hoge@example.com"));
		assert!(resp.is_ignore("hoge@fuga.com"));
		assert!(resp.is_ignore("hoge@fuga.pya.hoge.foresdon.jp"));
		assert!(!resp.is_ignore("kedama@foresdon.jp"));
	}

	fn data(responders: Vec<Box<dyn Responder>>) -> StatusProcessor {
		let config = serde_json::from_str::<Config>(DATA).unwrap();
		StatusProcessor {
			responders,
			keema: Keema::load().unwrap(),
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
