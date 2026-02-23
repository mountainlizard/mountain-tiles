use camino::Utf8PathBuf;

use crate::{
    app::App,
    data::{action::Action, modal::DataLossOperation, state::State},
    ui::file_dialog,
};

pub enum StateSource {
    NewOnAppStartup,
    New,
    Import { file_path: Utf8PathBuf },
    Open { file_path: Utf8PathBuf },
}

/// The contexts in which a document can be opened, e.g. used to
/// customise error messages
pub enum OpenContext {
    /// We're opening a file on startup, based on our stored save_path state,
    /// to start from the file that was open when application exited.
    AppStartup,

    /// User has requested to open a file using the UI
    UserAction,

    /// We're opening in response to a file argument, either received directly
    /// in args, or relayed to us via IPC from another instance that received
    /// in args, or on macOS from an open file event.
    FileArgument,
}

impl App {
    pub const MAXIMUM_RECENT_PATHS: usize = 10;

    pub fn push_recent_file_path(&mut self, path: Utf8PathBuf) {
        self.recent_paths.retain(|p| p != &path);
        self.recent_paths.push_front(path);
        while self.recent_paths.len() > Self::MAXIMUM_RECENT_PATHS {
            self.recent_paths.pop_back();
        }
    }

    pub fn on_open(&mut self, path: Utf8PathBuf) {
        self.save_path = Some(path.clone());
        self.push_recent_file_path(path.clone());
        self.mark_current_state_as_saved();
        self.update_texture_base_dir_from_file_path(Some(path));
    }

    pub fn on_save(&mut self, path: Utf8PathBuf) {
        self.save_path = Some(path.clone());
        self.push_recent_file_path(path.clone());
        self.act(Action::OnSave { path: path.clone() });
        self.update_texture_base_dir_from_file_path(Some(path));
        self.success("Saved");
    }

    pub fn clear_save_path(&mut self) {
        self.saved_revision = None;
        self.save_path = None;
        self.update_texture_base_dir_from_file_path(None);
    }

    /// Mark the current state as a saved revision. This means there will be
    /// no prompt for unsaved changes if the state is to be discarded (e.g.
    /// when using File->New, File->Open etc.)
    pub fn mark_current_state_as_saved(&mut self) {
        self.saved_revision = self.undo.most_recent_revision_index();
    }

    pub fn use_state(&mut self, state: State, source: StateSource) {
        let clear_edit_state = match source {
            StateSource::NewOnAppStartup => false,
            StateSource::New => true,
            StateSource::Import { .. } => true,
            StateSource::Open { .. } => true,
        };

        if clear_edit_state {
            self.edit = Default::default();
        }

        self.state = state;
        self.apply_invariants();
        self.select_first_layer();
        self.clear_undo();

        match source {
            StateSource::NewOnAppStartup => {
                self.clear_save_path();
                // Treat data as saved, since it contains no user input
                self.mark_current_state_as_saved();
            }

            StateSource::New => {
                self.clear_save_path();
                // Treat data as saved, since it contains minimal user input, and
                // it may be confuding/annoying to be prompted for unsaved data if
                // only a blank new document is present
                self.mark_current_state_as_saved();
            }
            StateSource::Import { file_path } => {
                self.clear_save_path();
                // Special case - file is not saved, but we do have a valid base dir
                // using the imported file path
                self.update_texture_base_dir_from_file_path(Some(file_path));
            }
            StateSource::Open { file_path } => {
                self.on_open(file_path.clone());
            }
        }
    }

    pub fn new_document(&mut self) {
        self.use_state(State::default(), StateSource::New);
    }

    pub fn check_data_loss_then_new_document(&mut self) {
        if self.may_have_unsaved_changes() {
            self.show_data_loss_modal(DataLossOperation::New);
        } else {
            self.new_document();
        }
    }

    /// Attempt to open a document, display an error modal if this fails
    /// Returns true if a document was opened, false otherwise
    pub fn open_document(&mut self, path: Utf8PathBuf, context: OpenContext) -> bool {
        match State::from_path(path.clone()) {
            Ok(map) => {
                self.use_state(
                    map,
                    StateSource::Open {
                        file_path: path.clone(),
                    },
                );
                true
            }
            Err(e) => {
                let msg = match context {
                    OpenContext::AppStartup => format!(
                        "Failed to reopen file from last session:\n\n{}\n\n{}",
                        path, e
                    ),
                    OpenContext::UserAction => format!("Failed to open file:\n\n{}\n\n{}", path, e),
                    OpenContext::FileArgument => {
                        format!("Failed to open file:\n\n{}\n\n{}", path, e)
                    }
                };
                self.show_error_modal(&msg);
                false
            }
        }
    }

    /// Show modal to open a document, and if a file is selected, attempt to open that document.
    /// If either stage fails, show an error modal.
    /// Returns true if a document was opened, false otherwise. Note that false may indicate
    /// either that no file was selected, or that an error occurred selecting a file, or that
    /// the file could not be opened
    pub fn show_open_document_modal(&mut self) -> bool {
        match file_dialog::pick_mnp_file() {
            Ok(Some(path)) => self.open_document(path, OpenContext::UserAction),
            Ok(None) => false,
            Err(e) => {
                self.show_error_modal(&e.to_string());
                false
            }
        }
    }

    pub fn check_data_loss_then_show_open_document_modal(&mut self) {
        if self.may_have_unsaved_changes() {
            self.show_data_loss_modal(DataLossOperation::Open);
        } else {
            self.show_open_document_modal();
        }
    }

    pub fn check_data_loss_then_open_document_from_file_argument(&mut self, path: Utf8PathBuf) {
        if self.may_have_unsaved_changes() {
            self.show_data_loss_modal(DataLossOperation::OpenFileArgument { path });
        } else {
            self.open_document(path, OpenContext::FileArgument);
        }
    }

    pub fn show_save_document_modal(&mut self) {
        if let Some(ref path) = self.save_path {
            self.save_document(path.clone());
        } else {
            self.show_save_as_document_modal();
        }
    }

    pub fn show_save_as_document_modal(&mut self) {
        match file_dialog::save_mnp_file(&self.save_path) {
            Ok(Some(path)) => self.save_document(path),
            Ok(None) => {}
            Err(e) => self.show_error_modal(&e.to_string()),
        }
    }

    pub fn save_document(&mut self, path: Utf8PathBuf) {
        let saved_revision = self.undo.most_recent_revision_index();
        match self.state.save_to_path(path.clone()) {
            Ok(()) => {
                self.saved_revision = saved_revision;
                self.on_save(path);
            }
            Err(e) => self.show_error_modal(&e.to_string()),
        }
    }

    pub fn check_data_loss_then_quit(&mut self, ctx: &egui::Context) {
        if self.may_have_unsaved_changes() {
            self.show_data_loss_modal(DataLossOperation::Quit);
        } else {
            self.quit(ctx);
        }
    }

    pub fn quit(&mut self, ctx: &egui::Context) {
        self.quit_requested = true;
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }
}
