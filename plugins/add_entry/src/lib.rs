#[allow(dead_code)]
extern "C" {
	fn config_add_host(config: *mut pssh_models::SshConfig, host: *const pssh_models::Host);
    fn config_remove_host(config: *mut pssh_models::SshConfig, idx: usize) -> bool; 
    fn config_hosts_len(config: *mut pssh_models::SshConfig) -> usize; 
    fn config_get_host(config: *mut pssh_models::SshConfig, idx: usize, out_host: *mut pssh_models::Host) -> bool;
	fn create_settings_list() -> *mut pssh_models::OptionsMap;
	fn free_settings_list(v: *mut pssh_models::OptionsMap);

	fn list_create() -> *mut pssh_models::List;
	fn list_free(l: *mut pssh_models::List);
	fn list_push(l: *mut pssh_models::List, entry: pssh_models::ListEntry);
}

#[no_mangle]
extern "C" fn inspect_config(config: *mut pssh_models::SshConfig) {
	let other = unsafe { create_settings_list() };
    let name = "Additional";
    let host_name = "plugin.example.com"; 
	let host = pssh_models::Host {
		name: name.as_ptr().cast::<i8>(),
        name_len: name.len(),
		host_name: host_name.as_ptr().cast::<i8>(),
        host_name_len: host_name.len(),
		user: std::ptr::null(),
        user_len: 0,
		other,
	};
	unsafe { config_add_host(config, &host) };
	unsafe { free_settings_list(other) };
}

#[no_mangle]
unsafe extern "C" fn on_item_select(_host: *const pssh_models::Host) -> *mut pssh_models::List {
	std::ptr::null_mut()
}
