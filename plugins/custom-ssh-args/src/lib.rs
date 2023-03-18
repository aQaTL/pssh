use std::ffi::CStr;

#[allow(dead_code)]
extern "C" {
	fn add_host(list: *mut pssh_models::SshConfig, host: *const pssh_models::Host);
	fn create_settings_list() -> *mut pssh_models::OptionsMap;
	fn free_settings_list(v: *mut pssh_models::OptionsMap);

	fn list_create() -> *mut pssh_models::List;
	fn list_free(l: *mut pssh_models::List);
	fn list_push(l: *mut pssh_models::List, entry: pssh_models::ListEntry);
}

#[no_mangle]
extern "C" fn inspect_config(_list: *mut pssh_models::SshConfig) {}

#[no_mangle]
unsafe extern "C" fn ssh_args(host: *const pssh_models::Host) -> *mut pssh_models::List {
	let host = &*host;
	let name = CStr::from_ptr(host.name).to_bytes();

	if name == b"Additional" {
		let args: [&[u8]; 2] = [b"cat\0", b"/etc/os-release\0"];

		let list = list_create();

		for arg in args {
			let entry = pssh_models::ListEntry {
				data: arg.as_ptr().cast(),
				len: arg.len(),
			};
			list_push(list, entry);
		}

		return list;
	}

	return std::ptr::null_mut();
}
