use std::{ffi::CString, os::windows::prelude::OsStrExt, path::Path};

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
	ssh_args_fn: SshArgsFn,
}

type InspectConfigFn = extern "C" fn(list: *mut pssh_sdk::SshConfig);
type SshArgsFn =
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

		let ssh_args_fn = unsafe { GetProcAddress(handle, b"ssh_args\0".as_ptr().cast::<i8>()) };
		if ssh_args_fn.is_null() {
			return Err(LoadError::LoadSshArgsFn(std::io::Error::last_os_error()));
		}
		let ssh_args_fn: SshArgsFn = unsafe { std::mem::transmute(ssh_args_fn) };

		Ok(Plugin {
			handle,
			inspect_config_fn,
			ssh_args_fn,
		})
	}

	pub fn call_inspect_config(&self, ssh_config: &mut ssh_config_parser::SshConfig) {
		let ssh_config: *mut ssh_config_parser::SshConfig = ssh_config;
		let ssh_config: *mut pssh_sdk::SshConfig = ssh_config.cast();
		(self.inspect_config_fn)(ssh_config);
	}

	pub fn call_ssh_args(&self, host: &ssh_config_parser::Host) -> Option<Vec<String>> {
		let name = format!("{}\0", host.name);
		let host_name = host
			.host_name
			.as_ref()
			.map(|host_name| format!("{}\0", host_name));
		let user = host.user.as_ref().map(|user| format!("{}\0", user));
		let host = pssh_sdk::pssh_models::Host {
			name: name.as_ptr().cast(),
			host_name: host_name
				.as_ref()
				.map(|x| x.as_ptr())
				.unwrap_or(std::ptr::null())
				.cast(),
			user: user
				.as_ref()
				.map(|x| x.as_ptr())
				.unwrap_or(std::ptr::null())
				.cast(),
			other: ((&host.other) as *const std::collections::HashMap<_, _>)
				.cast::<pssh_sdk::pssh_models::OptionsMap>(),
		};
		let list = (self.ssh_args_fn)(&host);
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
