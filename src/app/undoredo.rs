use crate::app::App;

impl App {
    pub fn can_undo(&self) -> bool {
        self.state.can_undo() && self.undo.has_undo(&self.state)
    }

    pub fn undo(&mut self) {
        if self.can_undo() {
            if let Some((undo_state, undo_edit)) = self.undo.undo(&self.state, &self.edit) {
                self.state = undo_state;
                self.edit.merge_undo_redo(undo_edit);
            }
        }
    }

    pub fn can_redo(&self) -> bool {
        self.state.can_redo() && self.undo.has_redo(&self.state)
    }

    pub fn redo(&mut self) {
        if self.can_redo() {
            if let Some((redo_state, redo_edit)) = self.undo.redo(&self.state) {
                self.state = redo_state;
                self.edit.merge_undo_redo(redo_edit);
            }
        }
    }

    /// Feed [`Undo`] with our current data and edit state
    pub(super) fn feed_undo(&mut self) {
        self.undo.feed_state(&self.state, &self.edit);
    }

    pub fn may_have_unsaved_changes(&self) -> bool {
        self.undo
            .may_have_changed_from_revision_index(self.saved_revision, &self.state)
    }
    // Create a new undo manager that will just have the current state,
    // with no undo states
    pub fn clear_undo(&mut self) {
        self.undo = Default::default();
        self.feed_undo();
    }
}
