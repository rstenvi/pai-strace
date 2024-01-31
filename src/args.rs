use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub enum Filter {
	#[default]
	None,
	Success,
	Fail,
}
impl std::str::FromStr for Filter {
	type Err = pai::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_lowercase().as_str() {
			"none" => Ok(Self::None),
			"success" => Ok(Self::Success),
			"fail" => Ok(Self::Fail),
			_ => Err(pai::Error::NotFound),
		}
	}
}
impl std::fmt::Display for Filter {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{:?}", self))
	}
}

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub enum Enrich {
	#[default]
	None,
	Basic,
	Full,
}
impl std::str::FromStr for Enrich {
	type Err = pai::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_lowercase().as_str() {
			"none" => Ok(Self::None),
			"basic" => Ok(Self::Basic),
			"full" => Ok(Self::Full),
			_ => Err(pai::Error::NotFound),
		}
	}
}
impl std::fmt::Display for Enrich {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{:?}", self))
	}
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum Format {
	#[default]
	Raw,
	Json,
}

impl std::str::FromStr for Format {
	type Err = pai::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_lowercase().as_str() {
			"raw" => Ok(Self::Raw),
			"json" => Ok(Self::Json),
			_ => Err(pai::Error::NotFound),
		}
	}
}
impl std::fmt::Display for Format {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{:?}", self))
	}
}

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
	#[command(flatten)]
	pub verbose: clap_verbosity_flag::Verbosity<clap_verbosity_flag::WarnLevel>,

	/// Attach to the argument, as opposed to spawning it
	#[arg(short, long)]
	pub attach: bool,

	/// Print output to file
	#[arg(short, long)]
	pub output: Option<PathBuf>,

	/// Attach to any new forked processes
	#[arg(long)]
	pub follow_childs: bool,

	/// Attach to any new forked processes
	#[arg(short, long)]
	pub follow_forks: bool,

	/// Only useful if output has been set, will create a new file per thread
	#[arg(long)]
	pub file_per_thread: bool,

	/// Only trace this comma-separeted list of system calls
	#[arg(long)]
	pub filter: Option<String>,

	/// Only report syscall enter and exit, you don't need this
	#[arg(long)]
	pub raw_mode: bool,

	#[arg(long)]
	pub print_stops: bool,

	#[arg(long)]
	pub print_events: bool,

	#[arg(long)]
	pub panic_on_oops: bool,

	#[arg(long)]
	pub check_update: bool,

	/// Enrich syscall data with more details about what the arguments are
	/// [None|Basic|Full].
	#[arg(long, default_value_t = Enrich::default())]
	pub enrich: Enrich,

	/// Only print, depending on outcome of syscall
	#[arg(long, default_value_t = Filter::default())]
	pub only_print: Filter,

	#[arg(long, default_values_t = [Format::default()])]
	pub format: Vec<Format>,

	/// Program to attach to, program to start, etc
	#[arg(trailing_var_arg = true, allow_hyphen_values = true)]
	pub args: Vec<String>,
}

