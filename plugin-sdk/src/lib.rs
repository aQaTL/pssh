use std::{collections::HashMap, ffi::CStr};

#[derive(Default, Debug, PartialEq, Eq)]
pub struct SshConfig {
	pub global_options: HashMap<String, String>,
	pub hosts: Vec<Host>,
}

#[derive(Default, Debug, PartialEq, Eq)]
pub struct Host {
	pub name: String,

	pub host_name: Option<String>,
	pub user: Option<String>,
	pub other: HashMap<String, String>,
}

type OptionsMap = HashMap<String, String>;

impl Host {
	unsafe fn from_c(host: *const plugin_models::Host) -> Self {
		let host = &*host;
		let name = CStr::from_ptr(host.name).to_str().unwrap().to_string();
		let host_name = if host.host_name.is_null() {
			None
		} else {
			Some(CStr::from_ptr(host.host_name).to_str().unwrap().to_string())
		};
		let user = if host.user.is_null() {
			None
		} else {
			Some(CStr::from_ptr(host.user).to_str().unwrap().to_string())
		};
		let other = (*host.other.cast::<OptionsMap>()).clone();
		Host {
			name,
			host_name,
			user,
			other,
		}
	}
}

#[no_mangle]
extern "C" fn add_host(list: *mut plugin_models::SshConfig, host: *const plugin_models::Host) {
	let ssh_config = unsafe { &mut *list.cast::<SshConfig>() };
	let host = unsafe { Host::from_c(host) };
	ssh_config.hosts.push(host);
}

#[no_mangle]
extern "C" fn create_settings_list() -> *mut plugin_models::OptionsMap {
	let options_map: Box<OptionsMap> = Box::new(HashMap::new());
	Box::into_raw(options_map).cast()
}

#[no_mangle]
unsafe extern "C" fn free_settings_list(options_map: *mut plugin_models::OptionsMap) {
	let _ = Box::from_raw(options_map.cast::<OptionsMap>());
}
