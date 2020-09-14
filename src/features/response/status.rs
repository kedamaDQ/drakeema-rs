use std::fs::File;
use std::io::BufReader;
use std::sync::mpsc;
use chrono:: { Local, Timelike };
use mastors::entities::{ Status, Visibility };
use regex::Regex;
use serde::Deserialize;
use crate::{
	Error,
	Monsters,
	Result,
	contents::*,
	utils::{
		transform_string_to_regex,
		transform_vec_string_to_vec_regex,
	},
};
use crate::message_processor::{
	Message,
	PollOptions,
};

const DATA: &str = "drakeema-data/features/response/status.json";
const TAG_P_REGEX: &str = r#"</?[pP][^>]*>"#;
const TAG_OTHER_REGEX: &str = r#"</?[^>]+>"#;

pub struct StatusProcessor {
	responders: Vec<Box<dyn Responder>>,
	keema: Keema,
	config: Config,
	tag_p_regex: Regex,
	tag_other_regex: Regex,
}

impl StatusProcessor {
	pub fn load() -> Result<Self> {
		use std::str::FromStr;

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
			tag_p_regex: Regex::from_str(TAG_P_REGEX)?,
			tag_other_regex: Regex::from_str(TAG_OTHER_REGEX)?,
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

		let text: Option<String>;
		let mut visibility = status.visibility();
		let mut mention = Some(status.account().acct().to_owned());
		let mut in_reply_to_id = Some(status.id().to_owned());
		let mut poll_options: Option<PollOptions> = None;

        if self.is_oshiete_keemasan(&content) {
            info!("Text matched keywords of Oshiete: {}", content);

			if status.account().is_local() && status.is_public() {
				in_reply_to_id = None;
			};

			let rc = ResponseCriteria::new(Local::now(), content);
            let mut t = self.responders.iter()
                .map(|i| i.respond(&rc))
                .filter(|i| i.is_some())
                .map(|i| i.unwrap())
                .collect::<Vec<String>>()
                .join("\n");

            if t.is_empty() {
                t = String::from("？");
            }

			text = Some(t);
		} else if self.is_keemasan(content) && self.is_healthcheck(content){
			info!("Text matched keywords of healthcheck: {}", content);

			text = self.config.healthcheck_responses.get(
				Local::now().second() as usize % self.config.healthcheck_responses.len()
			).map(|r| r.to_owned());
		} else if self.is_keemasan(&content) && self.can_i(content) {
			info!("Text matched keywords of Can I?: {}", content);

			let now = Local::now();
			if self.is_poll(now.second()) {
				info!("Get the special response of Can I?: {}", now);

				text = Some(self.format_for_poll(content));
				visibility = Visibility::Public;
				mention = None;
				in_reply_to_id = None;
				poll_options = Some(PollOptions::new(
					self.config.can_i_poll_options.clone(),
					self.config.can_i_poll_expires_in,
				));
			} else {
				text = self.config.can_i_responses.get(
					now.second() as usize % self.config.can_i_responses.len()
				)
				.map(|r| r.to_owned());
			}
        } else {
			text = self.keema.respond(&ResponseCriteria::new(Local::now(), content));
        }

        if let Some(text) = text {
			tx.send(Message::Status {
				text,
				visibility,
				mention,
				in_reply_to_id,
				poll_options,
			}).unwrap();
		}
	}

	fn is_ignore(&self, acct: &str) -> bool {
		self.config.ignore_acct_regex.iter().any(|re| re.is_match(acct))
	}

	fn is_healthcheck(&self, text: &str) -> bool {
		self.config.healthcheck_regex.is_match(text)
	}

	fn can_i(&self, text: &str) -> bool {
		self.config.can_i_regex.is_match(text)
	}

	fn is_poll(&self, sec: u32) -> bool {
		self.config.can_i_poll_secs.iter().any(|s| s == &sec)
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

	fn format_for_poll(&self, text: &str) -> String {
		let s = self.config.keemasan_regex.replace_all(text, "").to_string();
		let s = self.tag_p_regex.replace_all(&s, "\n").to_string();
		self.tag_other_regex.replace_all(&s, "").to_string()
	}
}

#[derive(Debug, Clone, Deserialize)]
struct Config {
    #[serde(deserialize_with = "transform_string_to_regex")]
	keemasan_regex: Regex,

    #[serde(deserialize_with = "transform_string_to_regex")]
	oshiete_regex: Regex,

	#[serde(deserialize_with = "transform_string_to_regex")]
	healthcheck_regex: Regex,
	healthcheck_responses: Vec<String>,

	#[serde(deserialize_with = "transform_string_to_regex")]
	can_i_regex: Regex,
	can_i_responses: Vec<String>,
	can_i_poll_secs: Vec<u32>,
	can_i_poll_options: Vec<String>,
	can_i_poll_expires_in: u64,

	#[serde(deserialize_with = "transform_vec_string_to_vec_regex")]
	ignore_acct_regex: Vec<Regex>,
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
		use std::str::FromStr;

		let config = serde_json::from_str::<Config>(DATA).unwrap();
		StatusProcessor {
			responders,
			keema: Keema::load().unwrap(),
			config,
			tag_p_regex: Regex::from_str(TAG_P_REGEX).unwrap(),
			tag_other_regex: Regex::from_str(TAG_OTHER_REGEX).unwrap(),
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
			"can_i_regex": "(?:いい|イイ|良い)(?:ですか|かな|の|かしら)*[!！]*[?？]",
			"can_i_responses": [
				"どうぞ！",
				"ダメです！",
				"しょうがない場合がある。",
				"許されない場合がある。",
				":x_exkun: へぇ〜！イイネ！",
				":x_ripo02: :x_dame:"
			],
			"can_i_poll_secs": [
				41
			],
			"can_i_poll_options": [
				"はい",
				"どちらかと言えばはい",
				"どちらかと言えばいいえ",
				"いいえ"
			],
			"can_i_poll_expires_in": 300,
			"ignore_acct_regex": [
				"@example.com$",
				"^hoge@fuga.com$",
				"hoge.foresdon.jp$"
			]
		}
	"#;
}
