use crate::commit::Commit;
use crate::constants::MINIMUM_FULL_WINDOW_WIDTH;
use crate::display::display_color::DisplayColor;
use crate::list::line::Line;
use crate::show_commit::show_commit_state::ShowCommitState;
use crate::show_commit::util::{get_diff_line_segments, get_stat_item_segments};
use crate::view::line_segment::LineSegment;
use crate::view::view_line::ViewLine;
use crate::view::View;
use std::cmp;
use unicode_segmentation::UnicodeSegmentation;

pub(super) struct Data {
	height: usize,
	width: usize,
	lines: Vec<ViewLine>,
	line_lengths: Vec<usize>,
	max_line_length: usize,
	state: ShowCommitState,
}

impl Data {
	pub(super) fn new() -> Self {
		Self {
			height: 0,
			width: 0,
			lines: Vec::new(),
			line_lengths: Vec::new(),
			max_line_length: 0,
			state: ShowCommitState::Overview,
		}
	}

	pub(super) fn reset(&mut self) {
		self.height = 0;
		self.width = 0;
		self.lines.clear();
		self.line_lengths.clear();
		self.max_line_length = 0;
	}

	pub(super) fn update(
		&mut self,
		state: &ShowCommitState,
		commit: &Commit,
		window_width: usize,
		window_height: usize,
	)
	{
		if window_width != self.width || window_height != self.height || *state != self.state {
			self.state = state.clone();
			self.reset();

			self.height = window_height;
			self.width = window_width;
			let is_full_width = window_width >= MINIMUM_FULL_WINDOW_WIDTH;
			match state {
				ShowCommitState::Diff => self.update_diff(commit, is_full_width),
				ShowCommitState::Overview => self.update_overview(commit, is_full_width),
			}
		}
	}

	fn update_diff(&mut self, commit: &Commit, is_full_width: bool) {
		match commit.get_file_stats() {
			Some(stats) => {
				for stat in stats {
					let old_largest_line_number_length = stat.largest_old_line_number().to_string().len();
					let new_largest_line_number_length = stat.largest_new_line_number().to_string().len();
					for delta in stat.deltas() {
						// TODO pad with line number segment
						let context = format!(
							"\n@@ -{},{} +{},{} @@ {}",
							delta.old_start(),
							delta.old_lines(),
							delta.new_start(),
							delta.new_lines(),
							delta.context()
						);
						self.lines.push(ViewLine::new(vec![LineSegment::new(context.as_str())]));
						self.line_lengths.push(context.len());

						for line in delta.lines() {
							// EOL support
							self.lines.push(ViewLine::new(get_diff_line_segments(
								line,
								old_largest_line_number_length,
								new_largest_line_number_length,
							)));
							self.line_lengths.push(
								line.line().len()
									+ old_largest_line_number_length + new_largest_line_number_length
									+ 4 + 7,
							);
						}
					}
				}
			},
			None => {},
		}
		// for f in commit.get_file_stats() {
		// 	// let new_largest_line_number_length = format!("{}", f.largest_new_line_number()).len();
		// 	// let old_largest_line_number_length = format!("{}", f.largest_old_line_number()).len();
		// 	// for d in f.deltas() {
		// 	// eprintln!(
		// 	// 	"\n@@ -{},{} +{},{} @@ {}",
		// 	// 	d.old_start(),
		// 	// 	d.old_lines(),
		// 	// 	d.new_start(),
		// 	// 	d.new_lines(),
		// 	// 	d.context()
		// 	// );
		// 	// // for l in d.lines() {
		// 	// eprint!("{} ", l.origin.to_str());
		// 	//
		// 	// if let Some(num) = l.old_line_number {
		// 	// 	eprint!("{:<width$}", num, width = old_largest_line_number_length);
		// 	// }
		// 	// else {
		// 	// 	eprint!("{:<width$}", "", width = old_largest_line_number_length);
		// 	// }
		// 	// eprint!(" * ");
		// 	// if let Some(num) = l.new_line_number {
		// 	// 	eprint!("{:>width$}", num, width = new_largest_line_number_length);
		// 	// }
		// 	// else {
		// 	// 	eprint!("{:>width$}", "", width = new_largest_line_number_length);
		// 	// }
		// 	// // TODO only show leading tabs/spaces
		// 	// // TODO configure show whitespace, true, false, leading, trailing, leading|trailing
		// 	// // TODO configure character to use for tab and space
		// 	// // TODO read and allow configuration of tab width
		// 	// eprint!("|  {}", l.line.replace(" ", "·").replace("\t", "→   "));
		// 	// }
		// 	// }
		// }
	}

	fn update_overview(&mut self, commit: &Commit, is_full_width: bool) {
		let full_hash = commit.get_hash();
		let author = commit.get_author();
		let committer = commit.get_committer();
		let date = commit.get_date();
		let body = commit.get_body();
		let file_stats = commit.get_file_stats();

		let hash_line = if is_full_width {
			format!("Commit: {}", full_hash)
		}
		else {
			let max_index = cmp::min(full_hash.len(), 8);
			format!("{:8} ", full_hash[0..max_index].to_string())
		};

		self.lines.push(ViewLine::new(vec![LineSegment::new_with_color(
			hash_line.as_str(),
			DisplayColor::IndicatorColor,
		)]));
		self.line_lengths.push(hash_line.len());

		let date_line = if is_full_width {
			format!("Date: {}", date.format("%c %z"))
		}
		else {
			format!("{}", date.format("%c %z"))
		};

		self.lines
			.push(ViewLine::new(vec![LineSegment::new(date_line.as_str())]));
		self.line_lengths.push(date_line.len());

		if let Some(a) = author.to_string() {
			let author_line = if is_full_width {
				format!("Author: {}", a)
			}
			else {
				format!("A: {}", a)
			};
			self.lines
				.push(ViewLine::new(vec![LineSegment::new(author_line.as_str())]));
			self.line_lengths
				.push(UnicodeSegmentation::graphemes(author_line.as_str(), true).count());
		}

		if let Some(c) = committer.to_string() {
			let committer_line = if is_full_width {
				format!("Committer: {}", c)
			}
			else {
				format!("C: {}", c)
			};
			self.lines
				.push(ViewLine::new(vec![LineSegment::new(committer_line.as_str())]));
			self.line_lengths
				.push(UnicodeSegmentation::graphemes(committer_line.as_str(), true).count());
		}

		match body {
			Some(b) => {
				for line in b.lines() {
					self.lines.push(ViewLine::new(vec![LineSegment::new(line)]));
					self.line_lengths
						.push(UnicodeSegmentation::graphemes(line, true).count());
				}
			},
			None => {},
		}

		self.lines.push(ViewLine::new(vec![LineSegment::new("")]));
		self.line_lengths.push(0);

		match file_stats {
			Some(stats) => {
				for stat in stats {
					let stat_to_name = stat.get_to_name();
					let stat_from_name = stat.get_from_name();
					let stat_view_line = ViewLine::new(get_stat_item_segments(
						stat.get_status(),
						stat_to_name.as_str(),
						stat_from_name.as_str(),
						is_full_width,
					));
					self.line_lengths.push(stat_view_line.get_length());
					self.lines.push(stat_view_line);
				}
			},
			None => {},
		}
	}

	pub(super) fn get_lines(&self) -> &Vec<ViewLine> {
		&self.lines
	}

	pub(super) fn get_max_line_length(&self, start: usize, end: usize) -> usize {
		let mut max_length = 0;
		for len in self.line_lengths[start..=end.min(self.line_lengths.len() - 1)].iter() {
			if *len > max_length {
				max_length = *len;
			}
		}
		max_length
	}
}
