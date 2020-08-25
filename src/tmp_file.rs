use std::fs;
use std::path::Path;

use crate::{
	Error,
	Result,
};

const TMP_PATH: &str = "drakeema-data/tmp/";

pub fn save_tmp(file: impl AsRef<str>, data: impl AsRef<str>) -> Result<()> {
	let path = TMP_PATH.to_string() + file.as_ref();
	info!("Start to save a temprary data: path: {}, data: {}", path, data.as_ref());

	let path = Path::new(&path);

	fs::write(path, data.as_ref().trim())
		.map_err(|e| Error::SaveTmpDataError(
			path.to_string_lossy().to_string(), e
		)
	)
}

pub fn load_tmp_as_string(file: impl AsRef<str>) -> Result<Option<String>> {
	let path = TMP_PATH.to_string() + file.as_ref();
	info!("Start to load a temporary data as string: {}", path);

	let path = Path::new(&path);

	if !path.exists() {
		return Ok(None)
	}

	Ok(Some(
		String::from_utf8_lossy(
			&fs::read(path)
				.map_err(|e|
					Error::LoadTmpDataError(path.to_string_lossy().to_string(), e
				)
			)?
		).trim().to_owned()
	))
}

pub fn load_tmp_as_i64(file: impl AsRef<str>) -> Result<Option<i64>> {
	match load_tmp_as_string(file) {
		Ok(t) => match t {
			Some(s) => {
				info!("Try to parse to i64: {}", s);
				Ok(Some(i64::from_str_radix(&s, 10)
					.map_err(|e| Error::TmpDataFormatError(s.to_owned(), e))?,
				))
			},
			None => Ok(None),
		},
		Err(e) => Err(e)
	}
}
