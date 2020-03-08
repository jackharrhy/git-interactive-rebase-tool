use crate::commit::diff_line::{DiffLine, Origin};
use crate::commit::status::Status;
use crate::display::display_color::DisplayColor;
use crate::list::line::Line;
use crate::view::line_segment::LineSegment;

pub(super) fn get_file_stat_color(status: &Status) -> DisplayColor {
	match status {
		Status::Added => DisplayColor::DiffAddColor,
		Status::Copied => DisplayColor::DiffAddColor,
		Status::Deleted => DisplayColor::DiffRemoveColor,
		Status::Modified => DisplayColor::DiffChangeColor,
		Status::Renamed => DisplayColor::DiffChangeColor,
		Status::Typechange => DisplayColor::DiffChangeColor,
		// this should never happen in a rebase
		Status::Other => DisplayColor::Normal,
	}
}

pub(super) fn get_file_stat_abbreviated(status: &Status) -> String {
	match status {
		Status::Added => String::from("A "),
		Status::Copied => String::from("C "),
		Status::Deleted => String::from("D "),
		Status::Modified => String::from("M "),
		Status::Renamed => String::from("R "),
		Status::Typechange => String::from("T "),
		// this should never happen in a rebase
		Status::Other => String::from("X "),
	}
}

pub(super) fn get_file_stat_long(status: &Status) -> String {
	match status {
		Status::Added => format!("{:>8}: ", "added"),
		Status::Copied => format!("{:>8}: ", "copied"),
		Status::Deleted => format!("{:>8}: ", "deleted"),
		Status::Modified => format!("{:>8}: ", "modified"),
		Status::Renamed => format!("{:>8}: ", "renamed"),
		Status::Typechange => format!("{:>8}: ", "changed"),
		// this should never happen in a rebase
		Status::Other => format!("{:>8}: ", "unknown"),
	}
}

pub(super) fn get_stat_item_segments(
	status: &Status,
	to_name: &str,
	from_name: &str,
	is_full_width: bool,
) -> Vec<LineSegment>
{
	let status_name = if is_full_width {
		get_file_stat_long(&status)
	}
	else {
		get_file_stat_abbreviated(&status)
	};

	let color = get_file_stat_color(&status);

	let to_file_indicator = if is_full_width { " -> " } else { ">" };

	match status {
		Status::Copied => {
			vec![
				LineSegment::new_with_color(status_name.as_str(), color),
				LineSegment::new_with_color(to_name, DisplayColor::Normal),
				LineSegment::new(to_file_indicator),
				LineSegment::new_with_color(from_name, DisplayColor::DiffAddColor),
			]
		},
		Status::Renamed => {
			vec![
				LineSegment::new_with_color(status_name.as_str(), color),
				LineSegment::new_with_color(to_name, DisplayColor::DiffRemoveColor),
				LineSegment::new(to_file_indicator),
				LineSegment::new_with_color(from_name, DisplayColor::DiffAddColor),
			]
		},
		_ => {
			vec![
				LineSegment::new_with_color(status_name.as_str(), color),
				LineSegment::new_with_color(from_name, color),
			]
		},
	}
}

pub(super) fn get_diff_line_segments(
	diff_line: &DiffLine,
	old_largest_line_number_length: usize,
	new_largest_line_number_length: usize,
) -> Vec<LineSegment>
{
	let mut line_segments = vec![];

	line_segments.push(LineSegment::new_with_color(
		diff_line.origin().to_str(),
		match diff_line.origin() {
			Origin::Addition => DisplayColor::DiffAddColor,
			Origin::Deletion => DisplayColor::DiffRemoveColor,
			Origin::Context => DisplayColor::Normal,
		},
	));

	line_segments.push(match diff_line.old_line_number() {
		Some(line_number) => {
			LineSegment::new(format!("{:<width$}", line_number, width = old_largest_line_number_length).as_str())
		},
		None => LineSegment::new(" ".repeat(old_largest_line_number_length).as_str()),
	});
	line_segments.push(LineSegment::new("   "));

	line_segments.push(match diff_line.new_line_number() {
		Some(line_number) => {
			LineSegment::new(format!("{:<width$}", line_number, width = new_largest_line_number_length).as_str())
		},
		None => LineSegment::new(" ".repeat(new_largest_line_number_length).as_str()),
	});
	// TODO only show leading tabs/spaces
	// TODO configure show whitespace, true, false, leading, trailing, leading|trailing
	// TODO configure character to use for tab and space
	// TODO read and allow configuration of tab width
	line_segments.push(LineSegment::new(
		format!("|  {}", diff_line.line().replace('\n', "").replace("\t", "→")).as_str(),
	));

	line_segments
}
