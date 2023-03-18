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
extern "C" fn inspect_config(list: *mut pssh_models::SshConfig) {
	let other = unsafe { create_settings_list() };
	let host = pssh_models::Host {
		name: b"Additional\0".as_ptr().cast::<i8>(),
		host_name: b"plugin.example.com\0".as_ptr().cast::<i8>(),
		user: std::ptr::null(),
		other,
	};
	unsafe { add_host(list, &host) };
	unsafe { free_settings_list(other) };
}

#[no_mangle]
unsafe extern "C" fn ssh_args(_host: *const pssh_models::Host) -> *mut pssh_models::List {
	std::ptr::null_mut()
}
