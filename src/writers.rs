use std::io::Write;

use pai::{
	api::messages::{Event, Stopped},
	syscalls::SyscallItem,
	utils::process::Tid,
};
use serde::{Deserialize, Serialize};
use struson::writer::JsonWriter;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawSyscall {
	tid: Tid,
	entry: bool,
}

impl RawSyscall {
	pub fn new(tid: Tid, entry: bool) -> Self {
		Self { tid, entry }
	}
}

impl std::fmt::Display for RawSyscall {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let entry = if self.entry { "entry" } else { "exit" };
		f.write_fmt(format_args!("{}: sycall({entry})", self.tid))
	}
}

pub struct WriteJson {
	writer: Option<struson::writer::JsonStreamWriter<Box<dyn std::io::Write>>>,
}
impl WriteJson {
	pub fn new(writer: struson::writer::JsonStreamWriter<Box<dyn std::io::Write>>) -> Self {
		Self {
			writer: Some(writer),
		}
	}
	fn writer(&mut self) -> &mut struson::writer::JsonStreamWriter<Box<dyn std::io::Write>> {
		self.writer.as_mut().expect("writer is None")
	}
}
pub struct WriteRaw {
	writer: Box<dyn Write>,
}
impl WriteRaw {
	pub fn new(writer: Box<dyn Write>) -> Self {
		Self { writer }
	}
}

pub trait SysWrite {
	fn init(&mut self) -> anyhow::Result<()>;
	fn write_raw_sysno(&mut self, raw: &RawSyscall) -> anyhow::Result<()>;
	fn write_syscall(&mut self, sys: &SyscallItem) -> anyhow::Result<()>;
	fn write_stop(&mut self, stop: &Stopped) -> anyhow::Result<()>;
	fn write_event(&mut self, event: &Event) -> anyhow::Result<()>;
	fn finish(&mut self) -> anyhow::Result<()>;
}
impl SysWrite for WriteRaw {
	fn init(&mut self) -> anyhow::Result<()> {
		Ok(())
	}

	fn write_raw_sysno(&mut self, raw: &RawSyscall) -> anyhow::Result<()> {
		let data = format!("{raw}\n");
		self.writer.write_all(data.as_bytes())?;
		Ok(())
	}

	fn write_syscall(&mut self, sys: &SyscallItem) -> anyhow::Result<()> {
		let data = format!("{sys}\n");
		self.writer.write_all(data.as_bytes())?;
		Ok(())
	}
	fn write_stop(&mut self, stop: &Stopped) -> anyhow::Result<()> {
		let data = format!("{stop}\n");
		self.writer.write_all(data.as_bytes())?;
		Ok(())
	}
	fn write_event(&mut self, event: &Event) -> anyhow::Result<()> {
		let data = format!("{event}\n");
		self.writer.write_all(data.as_bytes())?;
		Ok(())
	}
	fn finish(&mut self) -> anyhow::Result<()> {
		self.writer.flush()?;
		Ok(())
	}
}

impl SysWrite for WriteJson {
	fn init(&mut self) -> anyhow::Result<()> {
		self.writer().begin_array()?;
		Ok(())
	}
	fn write_raw_sysno(&mut self, raw: &RawSyscall) -> anyhow::Result<()> {
		self.writer().serialize_value(raw)?;
		Ok(())
	}
	fn write_syscall(&mut self, sys: &SyscallItem) -> anyhow::Result<()> {
		self.writer().serialize_value(sys)?;
		Ok(())
	}
	fn write_stop(&mut self, stop: &Stopped) -> anyhow::Result<()> {
		self.writer().serialize_value(stop)?;
		Ok(())
	}
	fn write_event(&mut self, event: &Event) -> anyhow::Result<()> {
		self.writer().serialize_value(event)?;
		Ok(())
	}
	fn finish(&mut self) -> anyhow::Result<()> {
		let mut writer =
			std::mem::take(&mut self.writer).expect("writer was None when trying to finish json");
		writer.end_array()?;
		writer.finish_document()?;
		Ok(())
	}
}
