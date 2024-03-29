mod args;
mod state;
mod writers;

use crate::args::{Args, Filter};
use anyhow::Result;
use clap::Parser;
use pai::api::messages::{CbAction, RegEvent, Stop};
use pai::ctx;

use crate::state::State;
use crate::writers::RawSyscall;

fn main() -> Result<()> {
	let mut args = Args::parse();
	pretty_env_logger::formatted_builder()
		.filter_level(args.verbose.log_level_filter())
		.init();

	log::info!("pai-strace started");
	args.init()?;
	args.sanity_check()?;

	let state = State::new(args.clone())?;
	let mut cargs = std::mem::take(&mut args.args);

	// Get main context object
	let prog = cargs.remove(0);
	let mut ctx: ctx::Main<State, anyhow::Error> =
		ctx::Main::new_main(args.attach, prog, cargs, state)?;

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
		sec.enrich_syscalls(args.enrich);
		sec.set_generic_syscall_handler(
			|cl, sys| {
				if cl.data().args.include_entry {
					cl.data_mut().write_syscall(sys.tid, sys)?;
				}
				Ok(CbAction::None)
			},
			|cl, sys| {
				let shouldprint = match cl.data().args.only_print {
					Filter::None => true,
					Filter::Success => sys.has_succeeded(),
					Filter::Fail => sys.has_failed(),
				};
				if shouldprint {
					cl.data_mut().write_syscall(sys.tid, sys)?;
				}
				Ok(CbAction::None)
			},
		);
	}

	if let Some(filter) = &args.filter {
		let mut conf = sec.take_args_builder();
		conf.set_intercept_all_syscalls(false);
		for part in filter.split(',') {
			let sysno = sec.client_mut().resolve_syscall(part)?;
			conf = conf.push_syscall_traced(sysno);
		}
		sec.set_args_builder(conf);
	}

	if !args.include_entry {
		sec.args_builder_mut().set_only_notify_syscall_exit(true);
	}
	if args.fix_ioctl_arg {
		sec.args_builder_mut().set_patch_ioctl_virtual(true);
	}

	if !args.follow_childs {
		sec.args_builder_mut().add_registered(RegEvent::Attached);
	}

	// We're all good and can just loop until program exits or we're detached.
	let (rsp, mut data) = ctx.loop_until_exit()?;
	log::debug!("final response {rsp:?}");
	data.finish()?;
	log::info!("done");
	Ok(())
}
