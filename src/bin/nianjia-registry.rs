use std::env;
use std::fs;

use clap::{Arg, App};
use nianjia::util::errors::NianjiaResult;
use nianjia::core::shell::Shell;

use registry::configuration::Configuration;

fn main() {
	let matches = App::new("nianjia-registry")
		.version(env!("CARGO_PKG_VERSION"))
		.author(env!("CARGO_PKG_AUTHORS"))
		.about(env!("CARGO_PKG_DESCRIPTION"))
		.arg(
			Arg::with_name("config")
				.short("c")
				.long("config")
				.value_name("FILE")
				.help("Sets a custom config file")
				.takes_value(true),
		)
		.get_matches();

	println!("{:?}", env::var_os("NIANJIA_HOME"));
	let config_file = matches.value_of("config").unwrap_or("default.conf");
	let _config =  match resolve_config(config_file) {
		Ok(cfg) => cfg,
		Err(e) => {
			let mut shell = Shell::new();
			nianjia::exit_with_error(e.into(), &mut shell)
		}
	};
}

fn resolve_config(file: &str) -> NianjiaResult<Configuration> {
	let content = fs::read_to_string(file)?;
	let config = serde_yaml::from_str(&content)?;
	Ok(config)
}