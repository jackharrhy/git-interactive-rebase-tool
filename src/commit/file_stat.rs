use crate::commit::delta::Delta;
use crate::commit::status::Status;
use crate::process::exit_status::ExitStatus::StateError;
use std::process::exit;

/// Represents a file change within a Git repository
#[derive(Debug)]
pub(crate) struct FileStat {
	status: Status,
	to_name: String,
	from_name: String,
	largest_old_line_number: u32,
	largest_new_line_number: u32,
	deltas: Vec<Delta>,
}

impl FileStat {
	/// Create a new FileStat
	pub(super) fn new() -> Self {
		FileStat {
			status: Status::Other,
			to_name: String::from(""),
			from_name: String::from(""),
			largest_old_line_number: 0,
			largest_new_line_number: 0,
			deltas: vec![],
		}
	}

	pub(super) fn reset(&mut self, from_name: String, to_name: String, status: Status) {
		self.status = status;
		self.from_name = from_name;
		self.to_name = to_name;
		self.largest_old_line_number = 0;
		self.largest_new_line_number = 0;
		self.deltas = vec![];
	}

	pub(super) fn new_from_existing(existing: &Self) -> Self {
		// there has to be a better way then this....
		let mut new = Self {
			status: existing.status.clone(),
			to_name: existing.to_name.clone(),
			from_name: existing.from_name.clone(),
			largest_old_line_number: existing.largest_old_line_number,
			largest_new_line_number: existing.largest_new_line_number,
			deltas: vec![],
		};

		for delta in &existing.deltas {
			new.deltas.push(Delta::new_from_existing(delta))
		}

		new
	}

	pub(super) fn add_delta(&mut self, delta: &Delta) {
		let last_old_line_number = delta.old_start() + delta.old_lines();
		if self.largest_old_line_number < last_old_line_number {
			self.largest_old_line_number = last_old_line_number;
		}
		let last_new_line_number = delta.new_start() + delta.new_lines();
		if self.largest_new_line_number < last_new_line_number {
			self.largest_new_line_number = last_new_line_number;
		}
		self.deltas.push(Delta::new_from_existing(delta));
	}

	/// Get the status of this file change
	pub(crate) fn get_status(&self) -> &Status {
		&self.status
	}

	/// Get the destination file name for this change.
	pub(crate) fn get_to_name(&self) -> &String {
		&self.to_name
	}

	/// Get the source file name for this change.
	pub(crate) fn get_from_name(&self) -> &String {
		&self.from_name
	}

	pub(crate) fn largest_old_line_number(&self) -> u32 {
		self.largest_old_line_number
	}

	pub(crate) fn deltas(&self) -> &Vec<Delta> {
		&self.deltas
	}

	pub(crate) fn largest_new_line_number(&self) -> u32 {
		self.largest_new_line_number
	}
}

#[cfg(test)]
mod tests {
	use crate::commit::file_stat::FileStat;
	use crate::commit::status::Status;

	#[test]
	fn commit_user_file_stat() {
		let file_stat = FileStat::new("/from/path".to_string(), "/to/path".to_string(), Status::Renamed);
		assert_eq!(*file_stat.get_status(), Status::Renamed);
		assert_eq!(file_stat.get_from_name(), "/from/path");
		assert_eq!(file_stat.get_to_name(), "/to/path");
	}
}
