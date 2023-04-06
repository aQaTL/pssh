#![no_std]

use core::ffi::c_void;

pub enum SshConfig {}

#[derive(Clone)]
#[repr(C)]
/// Strings are UTF-8, without null terminator. Not owned.
pub struct Host {
	pub name: *const i8,
	pub name_len: usize,

	pub host_name: *const i8,
	pub host_name_len: usize,
	pub user: *const i8,
	pub user_len: usize,
	pub other: *const OptionsMap,
}

pub enum OptionsMap {}

pub enum List {}

#[repr(C)]
pub struct ListEntry {
	pub data: *const c_void,
	pub len: usize,
}
