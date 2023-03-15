use std::collections::HashMap;

use nom::branch::alt;
use nom::bytes::complete::{tag, take_till, take_till1, take_until};
use nom::character::complete::{multispace0, multispace1, space1};
use nom::combinator::{all_consuming, map, value};
use nom::multi::many0;
use nom::sequence::{delimited, preceded, separated_pair, tuple};
use nom::IResult;

#[derive(Default, Debug, PartialEq, Eq)]
pub struct SshConfig {
	pub global_options: HashMap<String, String>,
	pub hosts: Vec<Host>,
}

#[derive(Default, Debug, PartialEq, Eq)]
pub struct Host {
	pub name: String,

	pub host_name: Option<String>,
	pub user: Option<String>,
	pub other: HashMap<String, String>,
}

pub fn parse(input: &str) -> Result<SshConfig, nom::Err<nom::error::Error<String>>> {
	enum Variant {
		Host(Host),
		GlobalOption(SshOption, String),
		Comment,
	}

	all_consuming(many0(delimited(
		multispace0,
		alt((
			map(parse_host, Variant::Host),
			map(recognize_comment, |_| Variant::Comment),
			map(parse_option, |(opt, value)| {
				Variant::GlobalOption(opt, value)
			}),
		)),
		multispace0,
	)))(input)
	.map(|(_input, variants): (_, Vec<Variant>)| {
		let mut global_options = HashMap::new();
		let mut hosts = Vec::new();
		for variant in variants {
			match variant {
				Variant::Host(host) => hosts.push(host),
				Variant::GlobalOption(kind, value) => {
					global_options.insert(kind.into(), value);
				}
				Variant::Comment => (),
			}
		}
		SshConfig {
			hosts,
			global_options,
		}
	})
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

		if let Ok((tail, _)) = recognize_comment(input) {
			input = tail;
			continue;
		}

		let result = parse_option(input);
		let (tail, (option_type, value)) = match result {
			Ok(v) => v,
			Err(_) => break,
		};
		input = tail;

		match option_type {
			SshOption::HostName => {
				host_name = Some(value);
			}
			SshOption::User => {
				user = Some(value);
			}
			SshOption::Other(name) => {
				other.insert(name, value);
			}
		}
	}

	Ok((
		rest,
		Host {
			name: name.to_string(),
			host_name,
			user,
			other,
		},
	))
}

// For internal usage
enum SshOption {
	HostName,
	User,
	Other(String),
}

impl From<SshOption> for String {
	fn from(v: SshOption) -> String {
		match v {
			SshOption::HostName => "HostName".to_string(),
			SshOption::User => "User".to_string(),
			SshOption::Other(v) => v,
		}
	}
}

fn parse_option(input: &str) -> IResult<&str, (SshOption, String)> {
	alt((
		map(
			preceded(
				tuple((tag("HostName"), space1)),
				take_till1(|c: char| c.is_whitespace()),
			),
			|v: &str| (SshOption::HostName, v.to_string()),
		),
		map(
			preceded(
				tuple((tag("User"), space1)),
				take_till1(|c: char| c.is_whitespace()),
			),
			|v: &str| (SshOption::User, v.to_string()),
		),
		map(
			separated_pair(
				take_till1(|c: char| c.is_whitespace()),
				space1::<&str, _>,
				take_till1(|c: char| c.is_whitespace()),
			),
			|(key, value)| (SshOption::Other(key.to_string()), value.to_string()),
		),
	))(input)
}

fn recognize_comment(input: &str) -> IResult<&str, ()> {
	value((), tuple((tag("#"), take_till(|c: char| c == '\n'))))(input)
}

#[cfg(test)]
mod tests {
	use std::collections::HashMap;

	use super::{Host, SshConfig};

	#[test]
	fn parse_single_host() {
		let single_host = "Host example_host\n\
            HostName example.com\n\
        	User example_user\n\
        ";

		let expected = Host {
			name: "example_host".to_string(),
			host_name: "example.com".to_string().into(),
			user: "example_user".to_string().into(),
			other: HashMap::new(),
		};

		let (_, actual) = super::parse_host(single_host).unwrap();
		assert_eq!(expected, actual);
	}

	#[test]
	fn parser_skips_comments() {
		let config = "
Host foo
    # This is my comment
    HostName example.com
    #User example_user
    User exampler
        ";

		let expected = SshConfig {
			hosts: vec![Host {
				name: "foo".to_string(),
				host_name: "example.com".to_string().into(),
				user: "exampler".to_string().into(),
				other: HashMap::new(),
			}],
			global_options: HashMap::new(),
		};

		let actual = super::parse(config).unwrap();
		assert_eq!(expected, actual);
	}

	#[test]
	fn global_options() {
		let config = "\n\
            StrictHostHeyChecking no\n\
            IdentityFile ~/.ssh/my_identity\n\
        ";

		let expected = SshConfig {
			global_options: [
				("StrictHostHeyChecking".to_string(), "no".to_string()),
				("IdentityFile".to_string(), "~/.ssh/my_identity".to_string()),
			]
			.into_iter()
			.collect(),
			hosts: Vec::new(),
		};

		let actual = super::parse(config).unwrap();
		assert_eq!(expected, actual);
	}

	#[test]
	fn global_comments() {
		let config = "# StrictHostHeyChecking no
#alamakota

Host foo
    HostName bar
#misformatted comment
    User foobar
    ";

		let expected = SshConfig {
			hosts: vec![Host {
				name: "foo".to_string(),
				host_name: "bar".to_string().into(),
				user: "foobar".to_string().into(),
				..Default::default()
			}],
			..Default::default()
		};

		let actual = super::parse(config).unwrap();
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
			global_options: HashMap::new(),
			hosts: vec![
				Host {
					name: "example_host".to_string(),
					host_name: "example.com".to_string().into(),
					user: "example_user".to_string().into(),
					other: HashMap::new(),
				},
				Host {
					name: "subexample".to_string(),
					host_name: "198.0.90.242".to_string().into(),
					user: "bob".to_string().into(),
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
