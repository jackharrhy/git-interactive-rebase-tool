use crate::display::Display;
use crate::input::utils::curses_input_to_string;
use crate::input::Input;
use crate::Config;
use pancurses::Input as PancursesInput;

#[derive(Debug, PartialEq)]
pub(crate) enum InputMode {
	List,
	ShowCommit,
	Other,
}

pub(crate) struct InputHandler<'i> {
	config: &'i Config,
	confirm_yes_input: char,
	display: &'i Display<'i>,
}

impl<'i> InputHandler<'i> {
	pub(crate) fn new(display: &'i Display, config: &'i Config) -> Self {
		let confirm_yes_input = config.input_confirm_yes.to_lowercase().chars().next().unwrap_or('y');
		Self {
			config,
			confirm_yes_input,
			display,
		}
	}

	#[allow(clippy::cognitive_complexity)]
	pub(crate) fn get_input(&self, mode: InputMode) -> Input {
		let c = self.get_next_input();

		let input = curses_input_to_string(c);

		match input.as_str() {
			i if i == self.config.input_abort.as_str() => Input::Abort,
			i if mode == InputMode::List && i == self.config.input_action_break.as_str() => Input::ActionBreak,
			i if mode == InputMode::List && i == self.config.input_action_drop.as_str() => Input::ActionDrop,
			i if i == self.config.input_help.as_str() => Input::Help,
			i if mode == InputMode::List && i == self.config.input_action_edit.as_str() => Input::ActionEdit,
			i if mode == InputMode::List && i == self.config.input_action_fixup.as_str() => Input::ActionFixup,
			i if mode == InputMode::List && i == self.config.input_action_pick.as_str() => Input::ActionPick,
			i if mode == InputMode::List && i == self.config.input_action_reword.as_str() => Input::ActionReword,
			i if mode == InputMode::List && i == self.config.input_action_squash.as_str() => Input::ActionSquash,
			i if mode == InputMode::List && i == self.config.input_edit.as_str() => Input::Edit,
			i if mode == InputMode::List && i == self.config.input_force_abort.as_str() => Input::ForceAbort,
			i if mode == InputMode::List && i == self.config.input_force_rebase.as_str() => Input::ForceRebase,
			i if i == self.config.input_move_down.as_str() => Input::MoveCursorDown,
			i if i == self.config.input_move_left.as_str() => Input::MoveCursorLeft,
			i if i == self.config.input_move_right.as_str() => Input::MoveCursorRight,
			i if i == self.config.input_move_selection_down.as_str() => Input::SwapSelectedDown,
			i if i == self.config.input_move_selection_up.as_str() => Input::SwapSelectedUp,
			i if i == self.config.input_move_up.as_str() => Input::MoveCursorUp,
			i if mode == InputMode::List && i == self.config.input_open_in_external_editor.as_str() => {
				Input::OpenInEditor
			},
			i if i == self.config.input_rebase.as_str() => Input::Rebase,
			i if mode == InputMode::List && i == self.config.input_show_commit.as_str() => Input::ShowCommit,
			i if i == self.config.input_show_diff.as_str() => Input::ShowDiff,
			i if mode == InputMode::List && i == self.config.input_toggle_visual_mode.as_str() => {
				Input::ToggleVisualMode
			},
			i if i == self.config.input_move_up_step.as_str() => Input::MoveCursorPageUp,
			i if i == self.config.input_move_down_step.as_str() => Input::MoveCursorPageDown,
			"Resize" => Input::Resize,
			_ => Input::Other,
		}
	}

	pub(crate) fn get_confirm(&self) -> Input {
		match self.display.getch() {
			Some(PancursesInput::Character(c)) => {
				if c.to_lowercase().next().unwrap() == self.confirm_yes_input {
					Input::Yes
				}
				else {
					Input::No
				}
			},
			Some(PancursesInput::KeyResize) => Input::Resize,
			_ => Input::No,
		}
	}

	pub(crate) fn get_character(&self) -> Input {
		loop {
			let c = self.get_next_input();

			match c {
				PancursesInput::Character(c) if c == '\n' => break Input::Enter,
				PancursesInput::Character(c) => break Input::Character(c),
				PancursesInput::KeyEnter => break Input::Enter,
				PancursesInput::KeyBackspace => break Input::Backspace,
				PancursesInput::KeyDC => break Input::Delete,
				PancursesInput::KeyRight => break Input::MoveCursorRight,
				PancursesInput::KeyLeft => break Input::MoveCursorLeft,
				PancursesInput::KeyResize => break Input::Resize,
				_ => {},
			};
		}
	}

	fn get_next_input(&self) -> PancursesInput {
		loop {
			let c = self.display.getch();
			if let Some(input) = c {
				break input;
			}
		}
	}
}
