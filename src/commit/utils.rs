use crate::commit::delta::Delta;
use crate::commit::diff_line::{DiffLine, Origin};
use crate::commit::file_stat::FileStat;
use crate::commit::status::Status;
use crate::commit::user::User;
use crate::commit::Commit;
use crate::process::exit_status::ExitStatus::StateError;
use chrono::{Local, TimeZone};
use git2::{DiffFindOptions, DiffOptions, Error, Repository};
use std::cmp::max;
use std::sync::Mutex;

/// Load commit information from a commit hash.
pub(super) fn load_commit_state(hash: &str) -> Result<Commit, Error> {
	let repo = Repository::open_from_env()?;
	let commit = repo.find_commit(repo.revparse_single(hash)?.id())?;

	let full_hash = commit.id().to_string();
	let date = Local.timestamp(commit.time().seconds(), 0);
	let body = commit.message().map(String::from);

	let author = User::new(commit.author().name(), commit.author().email());

	let committer = User::new(commit.committer().name(), commit.committer().email());

	let committer = if committer != author {
		committer
	}
	else {
		User::new(None, None)
	};

	let mut diff_options = DiffOptions::new();
	let diff_options = diff_options
		.include_unmodified(true)
		.show_unmodified(true)
		.ignore_filemode(false)
		.include_typechange_trees(true)
		.include_typechange(true)
		.minimal(true)
		.indent_heuristic(true)
		.context_lines(3);

	let mut diff_find_options = DiffFindOptions::new();
	let diff_find_options = diff_find_options
		.renames(true)
		.copies_from_unmodified(true)
		.copies(true)
		.remove_unmodified(true); // this doesn't seem to work

	// some commits do not have parents, and can't have file stats
	let file_stats = match commit.parent_ids().count() {
		0 => None,
		_ => {
			let mut diff = repo.diff_tree_to_tree(
				// parent exists from check above
				Some(&commit.parent(0)?.tree()?),
				Some(&commit.tree()?),
				Some(diff_options),
			)?;

			diff.find_similar(Some(diff_find_options))?;

			let mut stats: Vec<FileStat> = vec![];
			let mut file_stat = Mutex::new(FileStat::new());
			let mut delta = Mutex::new(Delta::new());

			// TODO trace file mode change and binary files
			diff.foreach(
				&mut |diffDelta, _| {
					// ignore unmodified files
					if diffDelta.status() == git2::Delta::Unmodified {
						return true;
					}

					eprintln!(
						"{}",
						diffDelta
							.old_file()
							.path()
							.map(|p| String::from(p.to_str().unwrap()))
							.unwrap_or_else(|| String::from("unknown")),
					);

					let mut fs_locked = file_stat.lock().unwrap();
					stats.push(FileStat::new_from_existing(&fs_locked));
					fs_locked.reset(
						diffDelta
							.old_file()
							.path()
							.map(|p| String::from(p.to_str().unwrap()))
							.unwrap_or_else(|| String::from("unknown")),
						diffDelta
							.new_file()
							.path()
							.map(|p| String::from(p.to_str().unwrap()))
							.unwrap_or_else(|| String::from("unknown")),
						Status::new_from_git_delta(diffDelta.status()),
					);
					true
				},
				None,
				Some(&mut |_, diffHunk| {
					let mut delta_locked = delta.lock().unwrap();
					if delta_locked.lines().len()
					file_stat.lock().unwrap().add_delta(&delta_locked);
					eprintln!(
						"{}, {}",
						file_stat.lock().unwrap().get_from_name(),
						delta_locked.context()
					);

					delta_locked.reset();
					delta_locked.set_context(
						// extra context from header
						std::str::from_utf8(diffHunk.header())
							.unwrap()
							.split("@")
							.last()
							.unwrap_or("")
							.trim(),
						diffHunk.old_start(),
						diffHunk.new_start(),
						diffHunk.old_lines(),
						diffHunk.new_lines(),
					);

					true
				}),
				Some(&mut |_, _, dl| {
					delta.lock().unwrap().add_line(DiffLine::new(
						Origin::from_chr(dl.origin()),
						std::str::from_utf8(dl.content()).unwrap(),
						dl.old_lineno(),
						dl.new_lineno(),
						dl.origin() == '=' || dl.origin() == '>' || dl.origin() == '<',
					));
					true
				}),
			)
			.unwrap();

			let mut fs_locked = file_stat.lock().unwrap();
			fs_locked.add_delta(&delta.lock().unwrap());
			stats.push(FileStat::new_from_existing(&fs_locked));
			// first element is always an empty delta, so remove it
			stats.remove(0);
			Some(stats)
		},
	};

	Ok(Commit::new(full_hash, author, committer, date, file_stats, body))
}

#[cfg(test)]
mod tests {
	// some of this file is difficult to test because it would require a non-standard git repo, so
	// we test what is possible
	use crate::commit::status::Status;
	use crate::commit::utils::load_commit_state;
	use serial_test::serial;
	use std::env::set_var;
	use std::path::Path;

	fn set_git_dir(fixture: &str) {
		set_var(
			"GIT_DIR",
			Path::new(env!("CARGO_MANIFEST_DIR"))
				.join("test")
				.join("fixtures")
				.join(fixture)
				.to_str()
				.unwrap(),
		);
	}

	#[test]
	#[serial]
	fn commit_utils_load_commit_state_load_hash() {
		set_git_dir("simple");
		let commit = load_commit_state("18d82dcc4c36cade807d7cf79700b6bbad8080b9").unwrap();
		assert_eq!(commit.hash, "18d82dcc4c36cade807d7cf79700b6bbad8080b9");
	}

	#[test]
	#[serial]
	fn commit_utils_load_commit_state_load_author() {
		set_git_dir("simple");
		let commit = load_commit_state("18d82dcc4c36cade807d7cf79700b6bbad8080b9").unwrap();
		assert_eq!(commit.get_author().to_string().unwrap(), "Tim Oram <dev@mitmaro.ca>");
	}

	#[test]
	#[serial]
	fn commit_utils_load_commit_state_load_date() {
		set_git_dir("simple");
		let commit = load_commit_state("18d82dcc4c36cade807d7cf79700b6bbad8080b9").unwrap();
		assert_eq!(commit.get_date().timestamp(), 1580172067);
	}

	#[test]
	#[serial]
	fn commit_utils_load_commit_state_load_body() {
		set_git_dir("simple");
		let commit = load_commit_state("18d82dcc4c36cade807d7cf79700b6bbad8080b9").unwrap();
		assert_eq!(
			commit.get_body().as_ref().unwrap(),
			"Empty commit title\n\nEmpty commit body\n"
		);
	}

	#[test]
	#[serial]
	fn commit_utils_load_commit_state_load_committer_match_author() {
		set_git_dir("simple");
		let commit = load_commit_state("ac950e31a96660e55d8034948b5d9b985c97692d").unwrap();
		assert!(commit.get_committer().to_string().is_none());
	}

	#[test]
	#[serial]
	fn commit_utils_load_commit_state_load_committer_not_match_author() {
		set_git_dir("simple");
		let commit = load_commit_state("2836dcdcbd040f9157652dd3db0d584a44d4793d").unwrap();
		assert_eq!(
			commit.get_committer().to_string().unwrap(),
			"Not Tim Oram <not-dev@mitmaro.ca>"
		);
	}

	#[test]
	#[serial]
	fn commit_utils_load_commit_state_load_modified_file() {
		set_git_dir("simple");
		let commit = load_commit_state("1cc0456637cb220155e957c641f483e60724c581").unwrap();
		let file_stat = commit.get_file_stats().as_ref().unwrap().first().unwrap();
		// 		file_stat.get_status()
		assert_eq!(*file_stat.get_status(), Status::Modified);
		assert_eq!(file_stat.get_from_name(), "a");
	}

	#[test]
	#[serial]
	fn commit_utils_load_commit_state_load_added_file() {
		set_git_dir("simple");
		let commit = load_commit_state("c1ac7f2c32f9e00012f409572d223c9457ae497b").unwrap();
		let file_stat = commit.get_file_stats().as_ref().unwrap().first().unwrap();
		// 		file_stat.get_status()
		assert_eq!(*file_stat.get_status(), Status::Added);
		assert_eq!(file_stat.get_from_name(), "e");
	}

	#[test]
	#[serial]
	fn commit_utils_load_commit_state_load_deleted_file() {
		set_git_dir("simple");
		let commit = load_commit_state("d85479638307e4db37e1f1f2c3c807f7ff36a0ff").unwrap();
		let file_stat = commit.get_file_stats().as_ref().unwrap().first().unwrap();
		// 		file_stat.get_status()
		assert_eq!(*file_stat.get_status(), Status::Deleted);
		assert_eq!(file_stat.get_from_name(), "b");
	}

	#[test]
	#[serial]
	fn commit_utils_load_commit_state_load_renamed_file() {
		set_git_dir("simple");
		let commit = load_commit_state("aed0fd1db3e73c0e568677ae8903a11c5fbc5659").unwrap();
		let file_stat = commit.get_file_stats().as_ref().unwrap().first().unwrap();
		// 		file_stat.get_status()
		assert_eq!(*file_stat.get_status(), Status::Renamed);
		assert_eq!(file_stat.get_from_name(), "c");
		assert_eq!(file_stat.get_to_name(), "f");
	}

	#[test]
	#[serial]
	fn commit_utils_load_commit_state_load_copied_file() {
		set_git_dir("simple");
		let commit = load_commit_state("c028f42bdb2a5a9f80adea23d95eb240b994a6c2").unwrap();
		let file_stat = commit.get_file_stats().as_ref().unwrap().first().unwrap();
		// 		file_stat.get_status()
		assert_eq!(*file_stat.get_status(), Status::Copied);
		assert_eq!(file_stat.get_from_name(), "d");
		assert_eq!(file_stat.get_to_name(), "g");
	}
}
