use std::fs;
use std::path::Path;

use crate::{
	Error,
	Result,
};

const TMP_PATH: &str = "drakeema-data/tmp/";

pub fn save_tmp(file: impl AsRef<str>, data: impl AsRef<str>) -> Result<()> {
	let path = TMP_PATH.to_string() + file.as_ref();
	let path = Path::new(&path);

	debug!("Start saving data to a temporary file: path: {:?}, data: {}", path, data.as_ref());

	fs::write(path, data.as_ref().trim())
		.map_err(|e| Error::SaveTmpDataError(
			path.to_string_lossy().to_string(), e
		)
	)?;

	debug!("Saving data to a temporary file is complete: path: {:?}", path);
	Ok(())
}

pub fn load_tmp_as_string(file: impl AsRef<str>) -> Result<Option<String>> {
	let path = TMP_PATH.to_string() + file.as_ref();
	let path = Path::new(&path);

	debug!("Start loading data as string from a temporary file: {:?}", path);

	if !path.exists() {
		debug!("File is nothing: {:?}", path);
		return Ok(None)
	}

	let data = String::from_utf8_lossy(
		&fs::read(path)
			.map_err(|e|
				Error::LoadTmpDataError(path.to_string_lossy().to_string(), e
			)
		)?
	).trim().to_owned();
	
	debug!("Loading data as string from a temporary file is complete: {:?}, data: {}", path, data);
	Ok(Some(data))
}

pub fn load_tmp_as_i64(file: impl AsRef<str>) -> Result<Option<i64>> {
	let path = format!("{}/{}", TMP_PATH, file.as_ref());
	info!("Start loading data as i64 from a temporary file: {}", path);

	match load_tmp_as_string(file) {
		Ok(t) => match t {
			Some(s) => {
				debug!("Parssing data as i64: {}", s);
				let data = i64::from_str_radix(&s, 10)
					.map_err(|e| Error::TmpDataFormatError(s.to_owned(), e))?;
				
				debug!("Loading data as i64 from a temporary file is complete: {}, data: {}", path, data);
				Ok(Some(data))
			},
			None => Ok(None),
		},
		Err(e) => Err(e)
	}
}
