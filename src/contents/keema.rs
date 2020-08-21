use std::fs::File;
use std::io::BufReader;
use serde::Deserialize;
use crate::{
	Error,
	Result,
};
use crate::features::{ Reaction, ReactionCriteria };
use crate::utils::transform_string_to_regex;

const DATA: &str = "data/contents/keema.json";

#[derive(Debug, Clone, Deserialize)]
pub struct Keema {
	keywords: Vec<Keyword>,
}

impl Keema {
    pub fn load() -> Result<Keema> {
		Ok(Keema {
			keywords: serde_json::from_reader(
				BufReader::new(File::open(DATA)?)
			)
			.map_err(|e| Error::ParseJsonError(DATA.to_owned(), e))?
		})
    }
}

impl Reaction for Keema {
	fn reaction(&self, criteria: &ReactionCriteria) -> Option<String> {
		use chrono::Timelike;

		self.keywords.iter()
			.find(|k| k.regex.is_match(criteria.text()))
			.map(|k| {
				k.reactions.get(
					criteria.at().second() as usize % k.reactions.len()
				)
				.unwrap()
				.to_owned()
			})
	}
}

#[derive(Debug, Clone, Deserialize)]
struct Keyword {
	#[serde(deserialize_with = "transform_string_to_regex")]
	regex: regex::Regex,
	reactions: Vec<String>,
}

#[cfg(test)]
mod tests {
	use super::*;
	use chrono::Local;

	#[test]
	fn test_is_match() {
		let keema = data();
		assert!(keema.reaction(&ReactionCriteria::new(Local::now(), "簡単なこと")).is_some());
	}

	#[test]
	fn test_is_not_match() {
		let keema = data();
		assert!(keema.reaction(&ReactionCriteria::new(Local::now(), "あいうえお")).is_none());
	}

	fn data() -> Keema {
		Keema {
			keywords: serde_json::from_str(DATA).unwrap()
		}
	}

	const DATA: &str = r#"
        [
        	{
        		"regex": "(?:簡単|かんたん|カンタン)な(?:事|こと|コト)",
        		"reactions": [
        			":x_ripo02: :x_dame:\n:x_ku02: とりあえずばつくれつけん！\n:x_pu02: よくわかんないけど悲しい…",
        			":x_gyousha: 簡単なことですよ️❤"
        		]
        	},
        	{
        		"regex": "[へヘﾍ][えエｴぇェｪ][ー〜]+[!！]*\\s*[いイｲ]{2}[ねネﾈ]",
        		"reactions": [
        			":x_ku02: ……ばくれちゅわ。",
        			":x_ripo02: :x_dame:"
        		]
        	},
        	{
        		"regex": "[欲ほ]しいな[あぁー〜]",
        		"reactions": [
        			":x_gyousha: お呼びですか？",
        			":x_exkun: でも全財産は1万7000ゴールド……"
        		]
        	},
        	{
        		"regex": "ばくれちゅ[わは]",
        		"reactions": [
        			":x_ku01: ばくれちゅわ！"
        		]
        	},
        	{
        		"regex": "(?:グローバルスター|ぐろーばるすたー)",
        		"reactions": [
        			":x_ripo01: 呼んだ？"
        		]
        	},
        	{
        		"regex": "(?:しめ|シメ|ｼﾒ)(?:さば|サバ|ｻﾊﾞ)",
        		"reactions": [
                  	":x_ku01: しめさばください！",
                  	":x_ku01: しめさばおいしい！",
                  	":x_ku01: しめさばは正義！"
        		]
        	},
        	{
        		"regex": "[うウｳ][ー〜]+[っッｯ]?(?:[くクｸ][っッｯ]){2,}",
        		"reactions": [
        			":c_porampan: ……うーくっくっくっ。"
        		]
        	},
        	{
        		"regex": "(?:ぺったんこ|ペッタンコ|ﾍﾟｯﾀﾝｺ)",
        		"reactions": [
                  	":c_anlucea: …………なにか言ったかしら💢",
                  	":c_anlucea: …………ぺったんこじゃないもん。"
        		]
        	},
        	{
        		"regex": "(?:でこ|デコ|ﾃﾞｺ)[っッｯ](?:ぱち|パチ|ﾊﾟﾁ)",
        		"reactions": [
                  	":c_anlucea: ‼️"
        		]
        	},
        	{
        		"regex": "[見み][付つっ]け(?:た|まし|ちゃ|てし)",
        		"reactions": [
                  	"いやー　さがしましたよ。"
        		]
        	},
        	{
        		"regex": "[捨す]て(?:ち|よう|る|て)",
        		"reactions": [
        			"それを捨てるなんてとんでもない！"
        		]
        	},
        	{
        		"regex": "[死し]ん(?:だ|でし|じゃっ)",
        		"reactions": [
                  	"しんでしまうとは　なにごとだ！",
                  	"しんでしまうとは　なんと　いなかものじゃ！",
                  	"へんじがない　ただのしかばねの場合がある。"
        		]
        	},
        	{
        		"regex": "(?:ルドマン|るどまん|ﾙﾄﾞﾏﾝ)",
        		"reactions": [
                  	"なんと　この私が　好きと申すか！？\n\n///"
        		]
        	}
        ]
	"#;
}
