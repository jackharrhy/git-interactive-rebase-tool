use crate::commit::diff_line::DiffLine;
use std::process::exit;

#[derive(Debug)]
pub(crate) struct Delta {
	old_start: u32,
	old_lines: u32,
	new_start: u32,
	new_lines: u32,
	context: String,
	lines: Vec<DiffLine>,
}

impl Delta {
	pub(super) fn new() -> Self {
		Self {
			old_start: 0,
			old_lines: 0,
			new_start: 0,
			new_lines: 0,
			context: String::from(""),
			lines: vec![],
		}
	}

	pub(super) fn new_from_existing(existing: &Self) -> Self {
		Self {
			old_start: existing.old_start,
			old_lines: existing.old_lines,
			new_start: existing.new_start,
			new_lines: existing.new_lines,
			context: String::from(existing.context.as_str()),
			lines: existing.lines.to_vec(),
		}
	}

	pub(super) fn reset(&mut self) {
		self.old_start = 0;
		self.old_lines = 0;
		self.new_start = 0;
		self.new_lines = 0;
		self.context = String::from("");
		self.lines = vec![];
	}

	pub(crate) fn context(&self) -> &str {
		self.context.as_str()
	}

	pub(super) fn set_context(&mut self, header: &str, old_start: u32, new_start: u32, old_lines: u32, new_lines: u32) {
		self.old_start = old_start;
		self.new_start = new_start;
		self.old_lines = old_lines;
		self.new_lines = new_lines;
		self.context = String::from(header);
	}

	pub(crate) fn lines(&self) -> &Vec<DiffLine> {
		&self.lines
	}

	pub(super) fn add_line(&mut self, diff_line: DiffLine) {
		self.lines.push(diff_line);
	}

	pub(crate) fn old_start(&self) -> u32 {
		self.old_start
	}

	pub(crate) fn old_lines(&self) -> u32 {
		self.old_lines
	}

	pub(crate) fn new_start(&self) -> u32 {
		self.new_start
	}

	pub(crate) fn new_lines(&self) -> u32 {
		self.new_lines
	}
	// 	pub(crate) fn to_string(&self) -> Option<String> {
	// 		let name = &self.name;
	// 		let email = &self.email;
	// 		match name {
	// 			Some(n) => {
	// 				match email {
	// 					Some(e) => Some(format!("{} <{}>", *n, *e)),
	// 					None => Some(n.to_string()),
	// 				}
	// 			},
	// 			None => {
	// 				match email {
	// 					Some(e) => Some(format!("<{}>", *e)),
	// 					None => None,
	// 				}
	// 			},
	// 		}
	// 	}
}

// #[derive(Debug)]
// pub(crate) struct DeltaBuilder {
// 	delta: Delta,
// }
//
// impl DeltaBuilder {
// 	pub(super) fn new() -> Self {
// 		Self {
// 			delta: Delta {
// 				header: None,
// 				lines: vec![],
// 			},
// 		}
// 	}
//
// 	pub(crate) fn header(mut self, header: &str) -> Self {
// 		self.delta.header = Some(String::from(header));
// 		self
// 	}
//
// 	pub(crate) fn line(mut self, line: DiffLine) -> Self {
// 		self.delta.lines.push(line);
// 		self
// 	}
//
// 	pub(crate) fn build(self) -> Delta {
// 		self.delta
// 	}
// }
