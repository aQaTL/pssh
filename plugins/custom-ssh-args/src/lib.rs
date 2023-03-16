use std::ffi::CStr;

#[allow(dead_code)]
extern "C" {
	fn add_host(list: *mut plugin_models::SshConfig, host: *const plugin_models::Host);
	fn create_settings_list() -> *mut plugin_models::OptionsMap;
	fn free_settings_list(v: *mut plugin_models::OptionsMap);

	fn list_create() -> *mut plugin_models::List;
	fn list_free(l: *mut plugin_models::List);
	fn list_push(l: *mut plugin_models::List, entry: plugin_models::ListEntry);
}

#[no_mangle]
extern "C" fn inspect_config(_list: *mut plugin_models::SshConfig) {}

#[no_mangle]
unsafe extern "C" fn ssh_args(host: *const plugin_models::Host) -> *mut plugin_models::List {
	let host = &*host;
	let name = CStr::from_ptr(host.name).to_bytes();

	if name == b"Additional" {
		let args: [&[u8]; 2] = [b"cat\0", b"/etc/os-release\0"];

		let list = list_create();

		for arg in args {
			let entry = plugin_models::ListEntry {
				data: arg.as_ptr().cast(),
				len: arg.len(),
			};
			list_push(list, entry);
		}

		return list;
	}

	return std::ptr::null_mut();
}
