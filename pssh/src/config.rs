use serde::Deserialize;
use std::{io, path::PathBuf};

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Config {
	pub launcher_cmd: Vec<String>,
	pub plugins: Vec<String>,
}

impl Default for Config {
	fn default() -> Self {
		Config {
			launcher_cmd: ["cmd.exe", "/c"]
				.into_iter()
				.map(ToString::to_string)
				.collect(),
			plugins: Vec::new(),
		}
	}
}

impl Config {
	pub fn load() -> Config {
		let path = match Config::file_path() {
			Ok(v) => v,
			Err(err) => {
				eprintln!("Failed to get config file path: {err:#?}");
				return Config::default();
			}
		};

		let file = match std::fs::read_to_string(path) {
			Ok(v) => v,
			Err(err) if err.kind() == io::ErrorKind::NotFound => {
				eprintln!("Using default config");
				return Config::default();
			}
			Err(err) => {
				eprintln!("Failed to load config file: {err:#?}");
				return Config::default();
			}
		};

		match toml::from_str::<Config>(&file) {
			Ok(config) => config,
			Err(err) => {
				eprintln!("Failed to parse config file: {err:#?}");
				Config::default()
			}
		}
	}

	fn file_path() -> io::Result<PathBuf> {
		let local_app_data = crate::local_app_data()?;
		let app_dir = local_app_data.join("pssh");
		if !app_dir.exists() {
			std::fs::create_dir(&app_dir)?;
		}
		let config_file = app_dir.join("config.toml");
		Ok(config_file)
	}
}
