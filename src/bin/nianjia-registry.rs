use std::env;

use clap::{App, Arg};

use nianjia::core::shell::Shell;

use registry::configuration::parse_file;

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
	match parse_file(config_file) {
		Ok(cfg) => {
			println!("{:?}", cfg);
		}
		Err(e) => {
			println!("{:?}", e);
			let mut shell = Shell::new();
			nianjia::exit_with_error(e.into(), &mut shell)
		}
	};
}

