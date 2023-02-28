use std::collections::HashMap;

use nom::bytes::complete::tag;
use nom::combinator::all_consuming;

#[derive(Default, Debug)]
pub struct SshConfig {
	pub hosts: Vec<Host>,
}

#[derive(Default, Debug)]
pub struct Host {
	pub name: String,

	pub host_name: String,
	pub user: String,
	pub other: HashMap<String, String>,
}

pub fn parse(input: &str) -> Result<SshConfig, nom::Err<nom::error::Error<String>>> {
	all_consuming(tag("foo"))(input)
		.map(|(_input, ssh_config)| todo!())
		.map_err(|err: nom::Err<nom::error::Error<&str>>| err.to_owned())
}
