extern "C" {
	fn add_host(list: *mut plugin_models::SshConfig, host: *const plugin_models::Host);
	fn create_settings_list() -> *mut plugin_models::OptionsMap;
	fn free_settings_list(v: *mut plugin_models::OptionsMap);
}

#[no_mangle]
extern "C" fn inspect_config(list: *mut plugin_models::SshConfig) {
	let other = unsafe { create_settings_list() };
	let host = plugin_models::Host {
		name: b"Additional\0".as_ptr().cast::<i8>(),
		host_name: b"plugin.example.com\0".as_ptr().cast::<i8>(),
		user: std::ptr::null(),
		other,
	};
	unsafe { add_host(list, &host) };
	unsafe { free_settings_list(other) };
}
