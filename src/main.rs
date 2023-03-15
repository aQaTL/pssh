#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::ffi::OsString;
use std::io;
use std::os::windows::prelude::OsStringExt;
use std::path::PathBuf;
use std::{cell::RefCell, ops::Deref, rc::Rc};

use native_windows_gui as nwg;
use nwg::NativeUi;

use config::Config;
use ssh_config_parser::SshConfig;

mod config;
mod ssh_config_parser;

fn main() {
	nwg::init().expect("Failed to init Native Windows GUI");
	nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");

	let config = Config::load();
	println!("Loaded config: {config:#?}");

	let _ui = App::build_ui(App {
		config,
		ssh_config: load_ssh_config_file().expect("Failed to load config file"),
		..Default::default()
	});
	nwg::dispatch_thread_events();
}

fn open_ssh_session(config: &Config, name: &str) {
	use winapi::um::winuser::{MessageBoxW, MB_ICONERROR};

	let result = std::process::Command::new(&config.launcher_cmd[0])
		.args(&config.launcher_cmd[1..])
		.arg(format!("ssh {name}"))
		.spawn();

	if let Err(err) = result {
		unsafe {
			MessageBoxW(
				std::ptr::null_mut(),
				err.to_string()
					.encode_utf16()
					.chain("\0".encode_utf16())
					.collect::<Vec<_>>()
					.as_ptr(),
				"Failed to start ssh\0"
					.encode_utf16()
					.collect::<Vec<_>>()
					.as_ptr(),
				MB_ICONERROR,
			);
		}
	}
}

#[derive(Default)]
pub struct App {
	window: nwg::Window,
	from_clipboard_button: nwg::Button,
	sessions_list_box: nwg::ListBox<String>,
	ip_input: nwg::TextInput,
	ok_button: nwg::Button,

	config: Config,
	ssh_config: SshConfig,
}

impl App {
	fn quit(&self) {
		nwg::stop_thread_dispatch();
	}

	fn paste_from_clipboard(&self) {
		let Some(text) = nwg::Clipboard::data_text(&self.window) else {
			return;
		};
		self.sessions_list_box.set_selection(None);
		self.ip_input.set_text(&text);
	}

	fn on_enter(&self) {
		if !self.ip_input.text().is_empty() && self.ip_input.focus() {
			self.open_from_custom_ip_input();
		} else {
			self.open_selected();
		}
	}

	fn on_ok_button(&self) {
		if self.ip_input.text().is_empty() {
			self.open_selected();
		} else {
			self.open_from_custom_ip_input();
		}
	}

	fn open_selected(&self) {
		let Some(selected_index) = self.sessions_list_box.selection() else {
			self.quit();
			return;
		};
		let item = &self.ssh_config.hosts[selected_index];
		println!("Selected index {selected_index}: {item:#?}");

		open_ssh_session(&self.config, &item.name);

		self.quit();
	}

	fn open_from_custom_ip_input(&self) {
		let ip = self.ip_input.text();
		println!("Opening from custom ip input: {ip}");

		open_ssh_session(&self.config, &ip);

		self.quit();
	}
}

struct AppUi {
	inner: Rc<App>,
	default_handler: RefCell<Option<nwg::EventHandler>>,
}

impl nwg::NativeUi<AppUi> for App {
	fn build_ui(mut data: Self) -> Result<AppUi, nwg::NwgError> {
		use nwg::{Event, EventData as Data, WindowFlags};

		nwg::Window::builder()
			.flags(WindowFlags::WINDOW | WindowFlags::VISIBLE)
			.size((200, 300))
			//.position((300, 300))
			.title("Pssh")
			.build(&mut data.window)?;

		// "From clipboard" button
		nwg::Button::builder()
			.size((180, 25))
			.position((10, 10))
			.text("From clipboard")
			.parent(&data.window)
			.build(&mut data.from_clipboard_button)?;

		// Sessions list box
		nwg::ListBox::builder()
			.size((180, 180))
			.position((10, 40))
			.focus(true)
			.collection(
				data.ssh_config
					.hosts
					.iter()
					.map(|host| host.name.clone())
					.collect(),
			)
			.parent(&mut data.window)
			.build(&mut data.sessions_list_box)?;

		// Ip input
		nwg::TextInput::builder()
			.size((180, 30))
			.position((10, 220))
			.placeholder_text(Some("Custom address"))
			.parent(&mut data.window)
			.build(&mut data.ip_input)?;

		// OK button
		nwg::Button::builder()
			.size((180, 30))
			.position((10, 250))
			.text("OK")
			.parent(&mut data.window)
			.build(&mut data.ok_button)?;

		let ui = AppUi {
			inner: Rc::new(data),
			default_handler: Default::default(),
		};

		let app = Rc::downgrade(&ui.inner);
		let event_handler = move |event, event_data, handle| {
			let Some(app) = app.upgrade() else {
				eprintln!("ERRO: Tried to handle event for deallocated App");
				return;
			};

			match event_data {
				Data::OnKey(nwg::keys::ESCAPE) => {
					app.quit();
				}
				Data::OnKey(nwg::keys::RETURN) => {
					app.on_enter();
				}
				_ => (),
			}

			match event {
				Event::OnButtonClick if &handle == &app.from_clipboard_button => {
					app.paste_from_clipboard();
				}
				Event::OnListBoxDoubleClick if &handle == &app.sessions_list_box => {
					app.on_enter();
				}
				Event::OnButtonClick if &handle == &app.ok_button => {
					app.on_ok_button();
				}
				Event::OnWindowClose if &handle == &app.window => {
					app.quit();
				}
				_ => (),
			}
		};

		*ui.default_handler.borrow_mut() = Some(nwg::full_bind_event_handler(
			&ui.inner.window.handle,
			event_handler,
		));

		Ok(ui)
	}
}

impl Drop for AppUi {
	fn drop(&mut self) {
		let Some(ref handler) = *self.default_handler.borrow_mut() else {
			return;
		};
		nwg::unbind_event_handler(handler);
	}
}

impl Deref for AppUi {
	type Target = App;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

fn load_ssh_config_file() -> Result<SshConfig, Box<dyn std::error::Error>> {
	let ssh_config = std::fs::read_to_string(home_dir()?.join(".ssh").join("config"))?;
	let config = ssh_config_parser::parse(&ssh_config)?;
	Ok(config)
}

fn home_dir() -> io::Result<PathBuf> {
	use winapi::shared::minwindef::MAX_PATH;
	use winapi::um::{
		processthreadsapi::{GetCurrentProcess, OpenProcessToken},
		userenv::GetUserProfileDirectoryW,
		winnt::{HANDLE, TOKEN_QUERY},
	};

	let mut token: HANDLE = std::ptr::null_mut();
	let ret =
		unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token as *mut HANDLE) };
	if ret == 0 {
		return Err(io::Error::last_os_error());
	}

	let mut path = [0u16; MAX_PATH];
	let mut path_size = MAX_PATH as u32;
	let ret = unsafe {
		GetUserProfileDirectoryW(token, &mut path as *mut u16, &mut path_size as *mut u32)
	};
	if ret == 0 {
		return Err(io::Error::last_os_error());
	}

	let path: PathBuf = OsString::from_wide(&path[..(path_size as usize - 1)]).into();
	Ok(path)
}

pub fn local_app_data() -> io::Result<PathBuf> {
	use winapi::um::{
		combaseapi::CoTaskMemFree, knownfolders::FOLDERID_LocalAppData,
		shlobj::SHGetKnownFolderPath,
	};

	let mut path_ptr: *mut u16 = std::ptr::null_mut();
	let ret = unsafe {
		SHGetKnownFolderPath(
			&FOLDERID_LocalAppData as *const _,
			0,
			std::ptr::null_mut(),
			&mut path_ptr as *mut *mut u16,
		)
	};
	if ret != 0 {
		return Err(io::Error::last_os_error());
	}

	let path_len = (0_usize..)
		.find(|offset| unsafe { *path_ptr.add(*offset) } == 0)
		.unwrap();
	let path: &[u16] = unsafe { std::slice::from_raw_parts_mut(path_ptr, path_len) };
	let path = OsString::from_wide(path);

	// Free the data allocated by Windows in SHGetKnownFolderPath
	unsafe { CoTaskMemFree(path_ptr.cast::<winapi::ctypes::c_void>()) }

	Ok(PathBuf::from(path))
}
