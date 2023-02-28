use std::ffi::OsString;
use std::io;
use std::os::windows::prelude::OsStringExt;
use std::path::PathBuf;
use std::{cell::RefCell, ops::Deref, rc::Rc};

use native_windows_gui as nwg;
use nwg::NativeUi;

mod ssh_config_parser;

fn main() {
	nwg::init().expect("Failed to init Native Windows GUI");
	nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");
	let _ui = App::build_ui(App {
		sessions_list: load_ssh_config_file().expect("Failed to load config file"),
		..Default::default()
	});
	nwg::dispatch_thread_events();
}

#[derive(Default)]
pub struct App {
	window: nwg::Window,
	from_clipboard_button: nwg::Button,
	sessions_list_box: nwg::ListBox<String>,
	ip_input: nwg::TextInput,
	ok_button: nwg::Button,

	sessions_list: Vec<String>,
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
		let item = &self.sessions_list[selected_index];
		println!("Selected index {selected_index}: {item}");

		self.quit();
	}

	fn open_from_custom_ip_input(&self) {
		let ip = self.ip_input.text();
		println!("Opening from custom ip input: {ip}");

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
			.collection(data.sessions_list.clone())
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

fn load_ssh_config_file() -> Result<Vec<String>, Box<dyn std::error::Error>> {
	let ssh_config = std::fs::read_to_string(home_dir()?.join(".ssh").join("config"))?;
	let config = ssh_config_parser::parse()?;

	Ok(vec![])
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
