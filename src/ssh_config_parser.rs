use std::collections::HashMap;

use nom::branch::alt;
use nom::bytes::complete::{tag, take_till1, take_until};
use nom::character::complete::{multispace0, multispace1, space1};
use nom::combinator::{all_consuming, map, value};
use nom::multi::many0;
use nom::sequence::{delimited, preceded, separated_pair, tuple};
use nom::IResult;

#[derive(Default, Debug, PartialEq, Eq)]
pub struct SshConfig {
	pub hosts: Vec<Host>,
}

#[derive(Default, Debug, PartialEq, Eq)]
pub struct Host {
	pub name: String,

	// TODO(aqatl): This should probably be optional
	pub host_name: String,
	// TODO(aqatl): This should probably be optional
	pub user: String,
	pub other: HashMap<String, String>,
}

pub fn parse(input: &str) -> Result<SshConfig, nom::Err<nom::error::Error<String>>> {
	all_consuming(many0(delimited(multispace0, parse_host, multispace0)))(input)
		.map(|(_input, hosts)| SshConfig { hosts })
		.map_err(|err: nom::Err<nom::error::Error<&str>>| err.to_owned())
}

fn parse_host(input: &str) -> IResult<&str, Host> {
	let (input, name) = preceded(
		tuple((tag("Host"), space1)),
		take_till1(|c: char| c.is_whitespace()),
	)(input)?;

	// Typing it out like that to avoid spelling out the error type in generic bounds
	let result: IResult<_, _> = take_until("Host ")(input);
	let (rest, input) = match result {
		Ok(v) => v,
		Err(_) => ("", input),
	};

	let mut host_name = None;
	let mut user = None;
	let mut other = HashMap::new();

	let mut input = input;

	loop {
		let result: IResult<_, _> = multispace1(input);
		let (tail, _) = match result {
			Ok(v) => v,
			Err(_) => break,
		};

		input = tail;
		let result: IResult<_, _> = alt((
			// this usage of value fn looks stupid
			value(
				(),
				map(
					preceded(
						tuple((tag("HostName"), space1)),
						take_till1(|c: char| c.is_whitespace()),
					),
					|v: &str| host_name = Some(v.to_string()),
				),
			),
			value(
				(),
				map(
					preceded(
						tuple((tag("User"), space1)),
						take_till1(|c: char| c.is_whitespace()),
					),
					|v: &str| user = Some(v.to_string()),
				),
			),
			value(
				(),
				map(
					separated_pair(
						take_till1(|c: char| c.is_whitespace()),
						space1::<&str, _>,
						take_till1(|c: char| c.is_whitespace()),
					),
					|(key, value)| other.insert(key.to_string(), value.to_string()),
				),
			),
		))(input);

		match result {
			Ok((tail, _)) => input = tail,
			Err(_) => break,
		}
	}

	Ok((
		rest,
		Host {
			name: name.to_string(),
			host_name: host_name.unwrap_or_default(),
			user: user.unwrap_or_default(),
			other,
		},
	))
}

#[cfg(test)]
mod tests {
	use std::collections::HashMap;

	use super::{Host, SshConfig};

	#[test]
	fn parse_single_host() {
		let single_host = "Host example_host
    HostName example.com
	User example_user\
    ";

		let expected = Host {
			name: "example_host".to_string(),
			host_name: "example.com".to_string(),
			user: "example_user".to_string(),
			other: HashMap::new(),
		};

		let (_, actual) = super::parse_host(single_host).unwrap();
		assert_eq!(expected, actual);
	}

	#[test]
	fn simple_config() {
		let config = "
Host example_host
    HostName example.com
	User example_user

Host subexample 
	HostName 198.0.90.242
	User bob
    Port 9082
    IdentityFile ~/.ssh/subexample.key

";

		let expected = SshConfig {
			hosts: vec![
				Host {
					name: "example_host".to_string(),
					host_name: "example.com".to_string(),
					user: "example_user".to_string(),
					other: HashMap::new(),
				},
				Host {
					name: "subexample".to_string(),
					host_name: "198.0.90.242".to_string(),
					user: "bob".to_string(),
					other: [
						("Port".to_string(), "9082".to_string()),
						(
							"IdentityFile".to_string(),
							"~/.ssh/subexample.key".to_string(),
						),
					]
					.into_iter()
					.collect(),
				},
			],
		};

		let actual = super::parse(config).unwrap();
		assert_eq!(expected, actual);
	}
}
