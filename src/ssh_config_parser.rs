use std::collections::HashMap;

use nom::bytes::complete::tag;
use nom::combinator::all_consuming;

struct SshConfig {
	hosts: Vec<Host>,
}

struct Host {
	name: String,

	host_name: String,
	user: String,
	other: HashMap<String, String>,
}

pub fn parse(input: &str) -> Result<SshConfig, nom::Err<nom::error::Error<String>>> {
	all_consuming(tag("foo"))(input)
		.map(|(_input, ssh_config)| todo!())
		.map_err(|err: nom::Err<nom::error::Error<&str>>| err.to_owned())
}
