use std::{
	collections::HashMap,
	fs::{File, OpenOptions},
	io::Write,
	path::PathBuf,
};

use pai::{
	api::messages::{Event, Stopped, SyscallItem},
	utils::process::Tid,
};
use struson::writer::JsonStreamWriter;

use crate::{
	args::{Args, Format},
	writers::{RawSyscall, SysWrite, WriteJson, WriteRaw},
};

macro_rules! gen_write_func {
	($name:ident, $arg:ident) => {
		pub fn $name(&mut self, tid: Tid, arg: &$arg) -> anyhow::Result<()> {
			// let tid = sys.tid;
			self.init_writer_tid(tid)?;

			for (_key, writer) in self.writers.iter_mut() {
				writer.$name(arg)?;
			}
			if let Some(writers) = self.tid_writers.get_mut(&tid) {
				for (_fmt, writer) in writers.iter_mut() {
					writer.$name(arg)?;
				}
			}
			Ok(())
		}
	};
}
pub struct State {
	pub args: Args,
	writers: HashMap<Format, Box<dyn SysWrite>>,
	tid_writers: HashMap<Tid, HashMap<Format, Box<dyn SysWrite>>>,
}
impl State {
	fn get_writer(path: &Option<PathBuf>, ext: &str) -> Box<dyn Write> {
		if let Some(path) = path {
			assert!(!path.is_dir());
			let mut path = path.clone();
			log::debug!("input file {path:?}");
			if !path.exists() {
				path.set_extension(ext);
			}
			log::info!("writing to {path:?}");
			let file = OpenOptions::new()
				.write(true)
				.create(true)
				.truncate(true)
				.open(path)
				.expect("Can't create output file");

			let r = Box::new(file);
			r as Box<dyn Write>
		} else {
			Self::get_stdout()
		}
	}
	fn get_writers(
		format: &[Format], output: &Option<PathBuf>,
	) -> anyhow::Result<HashMap<Format, Box<dyn SysWrite>>> {
		let mut writers = HashMap::new();
		for format in format.iter() {
			let mut ins = match format {
				Format::Json => {
					let writer = Self::get_writer(output, "json");
					let writer = struson::writer::JsonStreamWriter::new(writer);
					let writer = WriteJson::new(writer);
					Box::new(writer) as Box<dyn SysWrite>
				}
				Format::Raw => {
					let writer = Self::get_writer(output, "txt");
					Box::new(WriteRaw::new(writer)) as Box<dyn SysWrite>
				}
			};
			ins.init()?;
			writers.insert(format.clone(), ins);
		}
		Ok(writers)
	}
	pub fn new(args: Args) -> anyhow::Result<Self> {
		let tid_writers = HashMap::new();

		let writers = if !args.file_per_thread {
			Self::get_writers(&args.format, &args.output)?
		} else {
			HashMap::new()
		};

		let r = Self {
			args,
			writers,
			tid_writers,
		};
		Ok(r)
	}
	fn get_stdout() -> Box<dyn std::io::Write> {
		Box::new(std::io::stdout()) as Box<dyn std::io::Write + 'static>
	}
	fn out_name(&self, tid: Tid, ending: &str) -> String {
		if let Some(out) = &self.args.output {
			let s = out.as_os_str();
			let s = s.to_str().expect("unable to convert OsStr to str");
			format!("{s}_{tid}.{ending}")
		} else {
			todo!();
		}
	}
	fn init_writer_tid(&mut self, tid: Tid) -> anyhow::Result<()> {
		if self.args.file_per_thread && !self.tid_writers.contains_key(&tid) {
			let mut hins = HashMap::new();
			for format in self.args.format.iter() {
				let mut ins = match format {
					Format::Json => {
						let name = self.out_name(tid, "json");
						let r: Box<dyn Write> =
							Box::new(File::create(name).expect("Can't create file"));
						let writer = JsonStreamWriter::new(r);
						let writer = WriteJson::new(writer);
						Box::new(writer) as Box<dyn SysWrite>
					}
					Format::Raw => {
						let name = self.out_name(tid, "txt");
						let r: Box<dyn Write> =
							Box::new(File::create(name).expect("Can't create file"));
						let writer = WriteRaw::new(r);
						Box::new(writer)
					}
				};
				ins.init()?;
				hins.insert(format.clone(), ins);
			}
			self.tid_writers.insert(tid, hins);
		}
		Ok(())
	}

	gen_write_func! { write_raw_sysno, RawSyscall }
	gen_write_func! { write_syscall, SyscallItem }
	gen_write_func! { write_stop, Stopped }
	gen_write_func! { write_event, Event }

	pub fn finish(&mut self) -> anyhow::Result<()> {
		for (_key, mut writer) in std::mem::take(&mut self.writers).into_iter() {
			writer.finish()?;
		}
		for (_key, mut writers) in std::mem::take(&mut self.tid_writers).into_iter() {
			for (_fmt, writer) in writers.iter_mut() {
				writer.finish()?;
			}
		}
		Ok(())
	}
}
