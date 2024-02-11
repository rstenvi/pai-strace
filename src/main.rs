mod args;
mod state;
mod writers;

use crate::args::{Args, Enrich, Filter};
use anyhow::{Error, Result};
use clap::Parser;
use pai::api::messages::Stop;
use pai::api::ArgsBuilder;
use pai::ctx;
use pai::syscalls::Direction;

use crate::state::State;
use crate::writers::RawSyscall;

fn main() -> Result<()> {
	let mut args = Args::parse();
	pretty_env_logger::formatted_builder()
		.filter_level(args.verbose.log_level_filter())
		.init();

	log::info!("pai-strace started");
	if args.check_update {
		if let Ok(Some(version)) = check_latest::check_max!() {
			let msg = format!("version {version} is now available!");
			log::warn!("{msg}");
			log::warn!("update with 'cargo install --force pai-strace'");
			return Err(Error::msg(msg));
		} else {
			log::debug!("already running newest version");
		}
	}

	// Some sanity checking on the arguments.
	if args.only_print != Filter::None && args.enrich == Enrich::None {
		let msg = format!(
			"to filter on syscall result, you must enrich data with at least '{:?}'",
			Enrich::Basic
		);
		if args.panic_on_oops {
			return Err(Error::msg("no argument supplied, read help"));
		} else {
			log::warn!("{msg}");
		}
	}
	let state = State::new(args.clone())?;
	let cargs = std::mem::take(&mut args.args);
	if cargs.is_empty() {
		return Err(Error::msg("no argument supplied, read help"));
	}

	// Get main context object
	let mut ctx: ctx::Main<State, anyhow::Error> = ctx::Main::new_main(args.attach, cargs, state)?;

	// Start building our config based on the arguments
	let mut conf = ArgsBuilder::new();

	// First we setup all handler
	let sec = ctx.secondary_mut();
	sec.set_event_handler(|cl, evt| {
		if cl.data().args.print_events {
			let tid = evt.tid.unwrap_or(-1);
			cl.data_mut().write_event(tid, &evt)?;
		}
		Ok(())
	});
	sec.set_stop_handler(|cl, stop| {
		if cl.data().args.print_stops {
			cl.data_mut().write_stop(stop.tid, &stop)?;
		}
		if stop.stop == Stop::Attach && !cl.data().args.follow_childs {
			log::info!("detaching thread {}", stop.tid);
			cl.client_mut().detach_thread(stop.tid)?;
		}
		Ok(())
	});

	if args.raw_mode {
		sec.set_raw_syscall_handler(|cl, tid, entry| {
			let data = cl.data_mut();
			let p = RawSyscall::new(tid, entry);
			data.write_raw_sysno(tid, &p)?;
			Ok(())
		});
	} else {
		conf = conf.transform_syscalls();
		sec.set_generic_syscall_handler(|cl, mut sys| {
			if sys.is_exit() {
				let enrich = cl.data().args.enrich.clone();
				match enrich {
					Enrich::None => {}
					Enrich::Basic => sys.enrich_values()?,
					Enrich::Full => sys.parse_deep(sys.tid, cl.client_mut(), Direction::InOut)?,
				}
				let shouldprint = match cl.data().args.only_print {
					Filter::None => true,
					Filter::Success => sys.has_succeeded(),
					Filter::Fail => sys.has_failed(),
				};
				if shouldprint {
					cl.data_mut().write_syscall(sys.tid, &sys)?;
				}
			}
			Ok(())
		});
	}

	// All interactions which require a Client
	let client = ctx.secondary_mut().client_mut();

	// Whether we should print some or all syscalls
	if let Some(filter) = &args.filter {
		for part in filter.split(',') {
			let sysno = client.resolve_syscall(part)?;
			conf = conf.push_syscall_traced(sysno);
		}
	} else {
		conf = conf.intercept_all_syscalls();
	}

	// Finish up config and set it
	let conf = conf.finish()?;
	log::debug!("config {conf:?}");
	client.set_config(conf)?;

	// We're all good and can just loop until program exits or we're detached.
	let (rsp, mut data) = ctx.loop_until_exit()?;
	log::debug!("final response {rsp:?}");
	data.finish()?;
	log::info!("done");
	Ok(())
}
