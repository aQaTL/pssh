use std::{os::windows::prelude::OsStrExt, path::Path};

use crate::{config::Config, message_box_error, ssh_config_parser};

pub fn load_plugins(config: &Config) -> Vec<Plugin> {
	let mut plugins = Vec::with_capacity(config.plugins.len());
	for path in &config.plugins {
		match Plugin::load_from_dll(path.as_ref()) {
			Ok(v) => {
				println!("Loaded plugin {path}");
				plugins.push(v);
			}
			Err(err) => {
				if cfg!(debug_assertions) {
					eprintln!("Failed to load {path}: {err:?}");
				} else {
					message_box_error("Plugin load", &format!("Failed to load {path}: {err:?}"));
				}
			}
		}
	}
	plugins
}

use winapi::shared::minwindef::HMODULE;

pub struct Plugin {
	handle: HMODULE,
	inspect_config_fn: InspectConfigFn,
}

type InspectConfigFn = extern "C" fn(list: *mut plugin_sdk::SshConfig);

impl Drop for Plugin {
	fn drop(&mut self) {
		use winapi::um::libloaderapi::FreeLibrary;
		unsafe {
			let _ret = FreeLibrary(self.handle);
		}
	}
}

#[derive(Debug)]
pub enum LoadError {
	OsError(std::io::Error),
	LoadInspectConfigFn(std::io::Error),
}

impl Plugin {
	fn load_from_dll(path: &Path) -> Result<Self, LoadError> {
		use winapi::um::libloaderapi::{GetProcAddress, LoadLibraryW};

		let handle = unsafe {
			let path = path
				.as_os_str()
				.encode_wide()
				.chain(std::iter::once(0_u16))
				.collect::<Vec<_>>();
			LoadLibraryW(path.as_ptr())
		};

		if handle.is_null() {
			return Err(LoadError::OsError(std::io::Error::last_os_error()));
		}

		let inspect_config_fn =
			unsafe { GetProcAddress(handle, b"inspect_config\0".as_ptr().cast::<i8>()) };
		if inspect_config_fn.is_null() {
			return Err(LoadError::LoadInspectConfigFn(
				std::io::Error::last_os_error(),
			));
		}
		let inspect_config_fn: InspectConfigFn = unsafe { std::mem::transmute(inspect_config_fn) };

		Ok(Plugin {
			handle,
			inspect_config_fn,
		})
	}

	pub fn call_inspect_config(&self, ssh_config: &mut ssh_config_parser::SshConfig) {
		let ssh_config: *mut ssh_config_parser::SshConfig = ssh_config;
		let ssh_config: *mut plugin_sdk::SshConfig = ssh_config.cast();
		(self.inspect_config_fn)(ssh_config);
	}
}
