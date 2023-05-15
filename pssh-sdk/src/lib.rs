use std::{collections::HashMap, ffi::CStr};

pub use pssh_models;

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
	unsafe fn from_c(host: *const pssh_models::Host) -> Self {
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
pub unsafe extern "C" fn config_add_host(
	config: *mut pssh_models::SshConfig,
	host: *const pssh_models::Host,
) {
	let ssh_config = &mut *config.cast::<SshConfig>();
	let host = Host::from_c(host);
	ssh_config.hosts.push(host);
}

#[no_mangle]
pub extern "C" fn config_remove_host(config: *mut pssh_models::SshConfig, idx: usize) -> bool {
	let ssh_config: &mut SshConfig = unsafe { &mut *config.cast::<SshConfig>() };
	if ssh_config.hosts.get(idx).is_none() {
		return false;
	}
	ssh_config.hosts.remove(idx);
	true
}

#[no_mangle]
pub extern "C" fn config_hosts_len(config: *mut pssh_models::SshConfig) -> usize {
	let ssh_config: &mut SshConfig = unsafe { &mut *config.cast::<SshConfig>() };
	ssh_config.hosts.len()
}

#[no_mangle]
pub unsafe extern "C" fn config_get_host(
	config: *mut pssh_models::SshConfig,
	idx: usize,
	out_host: *mut pssh_models::Host,
) -> bool {
	let ssh_config: &mut SshConfig = unsafe { &mut *config.cast::<SshConfig>() };
	let Some(host) = ssh_config.hosts.get(idx) else {
        return false;
    };

	*out_host = pssh_models::Host {
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
		other: &host.other as *const _ as *const pssh_models::OptionsMap,
	};

	true
}

#[no_mangle]
pub extern "C" fn create_settings_list() -> *mut pssh_models::OptionsMap {
	let options_map: Box<OptionsMap> = Box::default();
	Box::into_raw(options_map).cast()
}

#[no_mangle]
pub unsafe extern "C" fn free_settings_list(options_map: *mut pssh_models::OptionsMap) {
	let _ = Box::from_raw(options_map.cast::<OptionsMap>());
}

#[derive(Default)]
pub struct List {
	pub v: Vec<Vec<u8>>,
}

#[no_mangle]
pub extern "C" fn list_create() -> *mut pssh_models::List {
	Box::into_raw(Box::<List>::default()).cast()
}

#[no_mangle]
pub unsafe extern "C" fn list_free(l: *mut pssh_models::List) {
	let _ = Box::from_raw(l.cast::<List>());
}

#[no_mangle]
pub unsafe extern "C" fn list_push(l: *mut pssh_models::List, entry: pssh_models::ListEntry) {
	let mut v = Vec::with_capacity(entry.len);
	std::ptr::copy_nonoverlapping(entry.data.cast::<u8>(), v.as_mut_ptr(), entry.len);
	v.set_len(entry.len);
	(*(l.cast::<List>())).v.push(v);
}
