use std::{ffi::CString, os::windows::prelude::OsStrExt, path::Path};

use crate::{config::Config, message_box_error};
use pssh_sdk::{Host, SshConfig};

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
	on_item_select_fn: OnItemSelectFn,
}

type InspectConfigFn = extern "C" fn(list: *mut pssh_sdk::SshConfig);
type OnItemSelectFn =
	extern "C" fn(host: *const pssh_sdk::pssh_models::Host) -> *mut pssh_sdk::pssh_models::List;

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
	LoadSshArgsFn(std::io::Error),
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

		let ssh_args_fn =
			unsafe { GetProcAddress(handle, b"on_item_select\0".as_ptr().cast::<i8>()) };
		if ssh_args_fn.is_null() {
			return Err(LoadError::LoadSshArgsFn(std::io::Error::last_os_error()));
		}
		let on_item_select_fn: OnItemSelectFn = unsafe { std::mem::transmute(ssh_args_fn) };

		Ok(Plugin {
			handle,
			inspect_config_fn,
			on_item_select_fn,
		})
	}

	pub fn call_inspect_config(&self, ssh_config: &mut SshConfig) {
		let ssh_config: *mut SshConfig = ssh_config;
		(self.inspect_config_fn)(ssh_config);
	}

	pub fn call_on_item_select(&self, host: &Host) -> Option<Vec<String>> {
		let host = pssh_sdk::pssh_models::Host {
			name: host.name.as_ptr().cast(),
			name_len: host.name.len(),

			host_name: host
				.host_name
				.as_ref()
				.map(|s| s.as_ptr().cast())
				.unwrap_or(std::ptr::null()),
			host_name_len: host.host_name.as_ref().map(|s| s.len()).unwrap_or_default(),

			user: host
				.user
				.as_ref()
				.map(|s| s.as_ptr().cast())
				.unwrap_or(std::ptr::null()),
			user_len: host.user.as_ref().map(|s| s.len()).unwrap_or_default(),

			other: &host.other as *const _ as *const pssh_sdk::pssh_models::OptionsMap,
		};
		let list = (self.on_item_select_fn)(&host);
		if list.is_null() {
			return None;
		}
		let list = unsafe { Box::from_raw(list.cast::<pssh_sdk::List>()) };
		Some(
			list.v
				.into_iter()
				.map(|bytes| {
					CString::from_vec_with_nul(bytes)
						.unwrap()
						.to_str()
						.unwrap()
						.to_string()
				})
				.collect(),
		)
	}
}
