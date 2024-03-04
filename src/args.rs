use std::path::PathBuf;

use clap::Parser;
use pai::api::args::Enrich;

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

	/// Create a new file per thread. Only useful if output has been set.
	#[arg(long)]
	pub file_per_thread: bool,

	/// Include syscall entry, default is to collect output and only print when
	/// exit is finished.
	#[arg(long)]
	pub include_entry: bool,

	/// Only trace this comma-separeted list of system calls
	#[arg(long, verbatim_doc_comment)]
	pub filter: Option<String>,

	/// Only report syscall enter and exit. You likely don't want to use this.
	#[arg(long, verbatim_doc_comment)]
	pub raw_mode: bool,

	/// Unstable
	#[arg(long)]
	pub print_stops: bool,

	/// Unstable
	#[arg(long)]
	pub print_events: bool,

	/// On startup we check for likely erroneous parameters. Default behaviour
	/// is to log a warning and continue. With this flag, program abort with
	/// error.
	#[arg(long, verbatim_doc_comment)]
	pub panic_on_oops: bool,

	/// Check for update when starting and exit if this is not the newest
	/// version.
	#[arg(long, verbatim_doc_comment)]
	pub check_update: bool,

	/// Fix ioctl() calls with added information about what `arg` argument
	/// points to based on the value of `cmd`. This parsing is based on the data
	/// we've parsed from Syzkaller and is best-effort.
	#[arg(long, verbatim_doc_comment)]
	pub fix_ioctl_arg: bool,

	/// Enrich syscall data with more details about what the arguments are
	/// [None|Basic|Full].
	#[arg(short, long, default_value_t = Enrich::default(), verbatim_doc_comment)]
	pub enrich: Enrich,

	/// Only print, depending on outcome of syscall
	#[arg(long, default_value_t = Filter::default(), verbatim_doc_comment)]
	pub only_print: Filter,

	/// Format for output, Raw or Json
	#[arg(long, default_values_t = [Format::default()], verbatim_doc_comment)]
	pub format: Vec<Format>,

	/// Program to attach to, program to start, etc
	#[arg(
		trailing_var_arg = true,
		allow_hyphen_values = true,
		verbatim_doc_comment
	)]
	pub args: Vec<String>,
}

impl Args {
	pub fn init(&self) -> anyhow::Result<()> {
		if self.check_update {
			if let Ok(Some(version)) = check_latest::check_max!() {
				let msg = format!("version {version} is now available!");
				log::warn!("{msg}");
				log::warn!("update with 'cargo install --force pai-strace'");
				Err(anyhow::Error::msg(msg))
			} else {
				log::debug!("already running newest version");
				Ok(())
			}
		} else {
			Ok(())
		}
	}
	/// A series of tests which may result in undesireable behaviour
	pub fn sanity_check(&self) -> anyhow::Result<()> {
		let mut oops = false;
		if let Some(first) = self.args.first() {
			if first.starts_with('-') {
				log::warn!(
					"target starts with '-' '{first:?}: you probably passed an invalid argument"
				);
				oops = true;
			}
		} else {
			// We cannot set `required` on `args` in clap because no argument is
			// valid in some cases, but not by the time we get here.
			log::warn!("no target supplied");
			return Err(anyhow::Error::msg(
				"Exiting because of inconsistent arguments",
			));
		}
		if self.attach && self.args.len() > 1 {
			log::warn!("only the first argument will be parsed when attaching, you provided multiple: {:?}", self.args);
			oops = true;
		}
		if self.only_print != Filter::None && self.enrich == Enrich::None {
			log::warn!(
				"to filter on syscall result, you must enrich data with at least '{:?}'",
				Enrich::Basic
			);
			oops = true;
		}
		if oops && self.panic_on_oops {
			Err(anyhow::Error::msg(
				"Exiting because of inconsistent arguments",
			))
		} else {
			Ok(())
		}
	}
}
