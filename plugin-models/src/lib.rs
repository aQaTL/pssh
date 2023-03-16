#![no_std]

pub enum SshConfig {}

#[derive(Clone)]
#[repr(C)]
pub struct Host {
	pub name: *const i8,

	pub host_name: *const i8,
	pub user: *const i8,
	pub other: *const OptionsMap,
}

pub enum OptionsMap {}
