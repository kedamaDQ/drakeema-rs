use std::fs::File;
use std::io::BufReader;
use serde::Deserialize;
use crate::{
	Error,
	Result,
};
use super::{ Responder, ResponseCriteria };
use crate::utils::transform_string_to_regex;

const DATA: &str = "drakeema-data/contents/keema.json";

#[derive(Debug, Clone, Deserialize)]
pub struct Keema {
	keywords: Vec<Keyword>,
}

impl Keema {
    pub fn load() -> Result<Keema> {
		info!("Initialize Keema");

		Ok(Keema {
			keywords: serde_json::from_reader(
				BufReader::new(File::open(DATA)?)
			)
			.map_err(|e| Error::ParseJsonError(DATA.to_owned(), e))?
		})
    }
}

impl Responder for Keema {
	fn respond(&self, criteria: &ResponseCriteria) -> Option<String> {
		use chrono::Timelike;

		debug!("Start building response from Keema: {:?}", criteria);

		let response = self.keywords.iter()
			.find(|k| k.regex.is_match(criteria.text()))
			.map(|k| {
				k.responses.get(
					criteria.at().second() as usize % k.responses.len()
				)
				.unwrap()
				.to_owned()
			});
		
		if response.is_some() {
			info!("Text matched keywords of Keema: {}", criteria.text());
		}

		response
	}
}

#[derive(Debug, Clone, Deserialize)]
struct Keyword {
	#[serde(deserialize_with = "transform_string_to_regex")]
	regex: regex::Regex,
	responses: Vec<String>,
}

#[cfg(test)]
pub(crate) mod tests {
	use super::*;
	use chrono::Local;

	#[test]
	fn test_is_match() {
		let keema = data();
		assert!(keema.respond(&ResponseCriteria::new(Local::now(), "簡単なこと")).is_some());
	}

	#[test]
	fn test_is_not_match() {
		let keema = data();
		assert!(keema.respond(&ResponseCriteria::new(Local::now(), "あいうえお")).is_none());
	}

	pub(crate) fn data() -> Keema {
		Keema {
			keywords: serde_json::from_str(DATA).unwrap()
		}
	}

	const DATA: &str = r#"
        [
        	{
        		"regex": "(?:簡単|かんたん|カンタン)な(?:事|こと|コト)",
        		"responses": [
        			":x_ripo02: :x_dame:\n:x_ku02: とりあえずばつくれつけん！\n:x_pu02: よくわかんないけど悲しい…",
        			":x_gyousha: 簡単なことですよ️❤"
        		]
        	},
        	{
        		"regex": "[へヘﾍ][えエｴぇェｪ][ー〜]+[!！]*\\s*[いイｲ]{2}[ねネﾈ]",
        		"responses": [
        			":x_ku02: ……ばくれちゅわ。",
        			":x_ripo02: :x_dame:"
        		]
        	},
        	{
        		"regex": "[欲ほ]しいな[あぁー〜]",
        		"responses": [
        			":x_gyousha: お呼びですか？",
        			":x_exkun: でも全財産は1万7000ゴールド……"
        		]
        	},
        	{
        		"regex": "ばくれちゅ[わは]",
        		"responses": [
        			":x_ku01: ばくれちゅわ！"
        		]
        	},
        	{
        		"regex": "(?:グローバルスター|ぐろーばるすたー)",
        		"responses": [
        			":x_ripo01: 呼んだ？"
        		]
        	},
        	{
        		"regex": "(?:しめ|シメ|ｼﾒ)(?:さば|サバ|ｻﾊﾞ)",
        		"responses": [
                  	":x_ku01: しめさばください！",
                  	":x_ku01: しめさばおいしい！",
                  	":x_ku01: しめさばは正義！"
        		]
        	},
        	{
        		"regex": "[うウｳ][ー〜]+[っッｯ]?(?:[くクｸ][っッｯ]){2,}",
        		"responses": [
        			":c_porampan: ……うーくっくっくっ。"
        		]
        	},
        	{
        		"regex": "(?:ぺったんこ|ペッタンコ|ﾍﾟｯﾀﾝｺ)",
        		"responses": [
                  	":c_anlucea: …………なにか言ったかしら💢",
                  	":c_anlucea: …………ぺったんこじゃないもん。"
        		]
        	},
        	{
        		"regex": "(?:でこ|デコ|ﾃﾞｺ)[っッｯ](?:ぱち|パチ|ﾊﾟﾁ)",
        		"responses": [
                  	":c_anlucea: ‼️"
        		]
        	},
        	{
        		"regex": "[見み][付つっ]け(?:た|まし|ちゃ|てし)",
        		"responses": [
                  	"いやー　さがしましたよ。"
        		]
        	},
        	{
        		"regex": "[捨す]て(?:ち|よう|る|て)",
        		"responses": [
        			"それを捨てるなんてとんでもない！"
        		]
        	},
        	{
        		"regex": "[死し]ん(?:だ|でし|じゃっ)",
        		"responses": [
                  	"しんでしまうとは　なにごとだ！",
                  	"しんでしまうとは　なんと　いなかものじゃ！",
                  	"へんじがない　ただのしかばねの場合がある。"
        		]
        	},
        	{
        		"regex": "(?:ルドマン|るどまん|ﾙﾄﾞﾏﾝ)",
        		"responses": [
                  	"なんと　この私が　好きと申すか！？\n\n///"
        		]
        	}
        ]
	"#;
}
