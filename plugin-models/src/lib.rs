#![no_std]

use core::ffi::c_void;

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

pub enum List {}

#[repr(C)]
pub struct ListEntry {
	pub data: *const c_void,
	pub len: usize,
}
