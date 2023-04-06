use std::mem::MaybeUninit;

#[allow(dead_code)]
extern "C" {
	fn config_add_host(config: *mut pssh_models::SshConfig, host: *const pssh_models::Host);
	fn config_remove_host(config: *mut pssh_models::SshConfig, idx: usize) -> bool;
	fn config_hosts_len(config: *mut pssh_models::SshConfig) -> usize;
	fn config_get_host(
		config: *mut pssh_models::SshConfig,
		idx: usize,
		out_host: *mut pssh_models::Host,
	) -> bool;
	fn create_settings_list() -> *mut pssh_models::OptionsMap;
	fn free_settings_list(v: *mut pssh_models::OptionsMap);

	fn list_create() -> *mut pssh_models::List;
	fn list_free(l: *mut pssh_models::List);
	fn list_push(l: *mut pssh_models::List, entry: pssh_models::ListEntry);
}

#[allow(dead_code)]
#[derive(Debug)]
struct Host<'a> {
    name: &'a str,

    host_name: Option<&'a str>,
    user: Option<&'a str>,
}

#[no_mangle]
unsafe extern "C" fn inspect_config(config: *mut pssh_models::SshConfig) {
	let hosts_len = config_hosts_len(config);

	for idx in 0..hosts_len {
		let mut host = MaybeUninit::<pssh_models::Host>::uninit();
		if !config_get_host(config, idx, host.as_mut_ptr()) {
			eprintln!("Failed to get idx {idx}");
			continue;
		}
		let host = host.assume_init();

		let name = std::str::from_utf8_unchecked(std::slice::from_raw_parts(
			host.name.cast(),
			host.name_len,
		));
		let host_name: Option<&str> = if host.host_name.is_null() {
			None
		} else {
			std::str::from_utf8_unchecked(std::slice::from_raw_parts(
				host.host_name.cast(),
				host.host_name_len,
			))
			.into()
		};
        let user: Option<&str> = if host.user.is_null() {
			None
		} else {
			std::str::from_utf8_unchecked(std::slice::from_raw_parts(
				host.user.cast(),
				host.user_len,
			))
			.into()
		};

        let h = Host {
            name,
            host_name,
            user,
        };
        println!("{idx}: {h:#?}");
	}
}

#[no_mangle]
unsafe extern "C" fn on_item_select(_host: *const pssh_models::Host) -> *mut pssh_models::List {
	std::ptr::null_mut()
}
