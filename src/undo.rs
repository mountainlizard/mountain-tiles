use std::collections::VecDeque;

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Settings {
    /// Maximum number of undos.
    /// If your state is resource intensive, you should keep this low.
    ///
    /// Default: `200`
    pub max_undos: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Self { max_undos: 200 }
    }
}

/// An item of data that can detect changes, and be cloned, allowing it to be tracked by [`Undo`]
pub trait Undoable: Clone {
    /// Check whether this data has changed from a previous state.
    fn has_changed_from(&self, previous: &Self) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RevisionIndex(u64);

impl RevisionIndex {
    pub const FIRST: RevisionIndex = RevisionIndex(1);

    pub fn next(&self) -> RevisionIndex {
        RevisionIndex(self.0 + 1)
    }

    pub fn index(&self) -> u64 {
        self.0
    }
}

/// A revision of the data that we can undo/redo to
/// Stores both data, and a view state, so we can handle undo/redo appropriately
/// for each.
/// Only changes to the data ever trigger a revision to be created, when
/// [`Undoable::has_changed_from`] is true for a comparison of the current
/// data to the previous revision's data
/// Say the data consists of a [`Vec`] of strings - in this case the
/// view data might be the range of indices of the [`Vec`] that are displayed
/// to the user. When we undo or redo an edit, we can then set the view to the range
/// that was in effect when the edit was made, which may make it easier for the
/// user to see the effect of the edit.
/// If this isn't desired, put all the data including view settings in `data`, and
/// use `()` for the view - values can then be ignored.
/// Note that `view` does not need to contain all view state - just the state
/// that should be controlled by the [`Undo`] system. Typically this would be
/// state that controls which data is visible. So for example in a drawing package,
/// we might store which image tab is selected, and the zoom settings for each image,
/// but not which drawing tool or color is selected.
#[derive(Debug, Clone)]
struct Revision<D: Undoable, V: Clone> {
    index: RevisionIndex,
    data: D,
    view: V,
}

/// Undo system.
///
/// When a change may have been made, feed it the most recent data and view state.
///
/// The [`Undo`] compares the data with the latest [`Revision`]
/// and if there is a change it will create a new [`Revision`].
///
/// Changes to data are checked using a trait [`Undoable`], so you can
/// provide custom logic, for example by ignoring unimportant or transient changes.
#[derive(Clone)]
pub struct Undo<D: Undoable, V: Clone> {
    settings: Settings,

    /// New undoes are added to the back.
    /// Two adjacent undo points are never equal.
    /// The latest undo point may (often) be the current state.
    undos: VecDeque<Revision<D, V>>,

    /// Stores redos immediately after a sequence of undos.
    /// Gets cleared every time the state changes.
    /// Does not need to be a deque, because there can only be up to `undos.len()` redos,
    /// which is already limited to `settings.max_undos`.
    redos: Vec<Revision<D, V>>,

    /// The next [`Revision`] index that will be assigned. Indices just need
    /// to be unique for revisions within this [`Undo`], no other properties
    /// are needed/guaranteed.
    next_revision_index: RevisionIndex,
}

impl<D: Undoable, V: Clone> std::fmt::Debug for Undo<D, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { undos, redos, .. } = self;
        f.debug_struct("Undo")
            .field("undo count", &undos.len())
            .field("redo count", &redos.len())
            .finish()
    }
}

impl<D: Undoable, V: Clone> Default for Undo<D, V> {
    #[inline]
    fn default() -> Self {
        Self {
            settings: Settings::default(),
            undos: VecDeque::new(),
            redos: Vec::new(),
            next_revision_index: RevisionIndex::FIRST,
        }
    }
}

impl<D: Undoable, V: Clone> Undo<D, V> {
    /// The index of the most recent revision recorded. Note that the state of the current
    /// data might have changed since this revision was made, if not all changes are recorded
    /// as revisions due to the implementation of [`Undoable`] (more details later).
    ///
    /// This allows for tracking whether data has changed relative to file contents:
    ///
    /// 1. When new data is created, store a value of `None` for the `saved_revision`
    ///    of that data.
    /// 2. When a file is opened to produce data, pass this data to [`Undo`] to create an
    ///    initial revision, and then call [`Undo::most_recent_revision_index`]
    ///    and store the result as the `saved_revision` for the data.
    /// 3. When data is saved to a file, call [`Undo::most_recent_revision_index`]
    ///    and store the result as the `saved_revision` for the data.
    /// 4. When you need to know whether data has unsaved changes, call
    ///    [`Undo::has_changed_from_revision_index`] with the `saved_revision` value and
    ///    current data. This will return true if the data _may have_ changed relative
    ///    to the contents of the file it was most recently opened from or saved to.
    ///
    /// Note that in the case where not all data changes result in a new revision (e.g.
    /// due to the implementation of [`Undoable`]) for the data type), this process still
    /// works but may sometimes report that data has changed when it hasn't (note this is
    /// the "safe" behaviour since it shouldn't lead to data loss, just a prompt to save
    /// when it may not be necessary).
    ///
    /// The mechanism by which this can occur is:
    ///
    /// 1. A data change is made that results in a new revision, e.g. with index 10.
    /// 2. At least one change is made that does not result in a new revision.
    /// 3. The data is saved, [`Undo::most_recent_revision_index`] is called, and
    ///    `saved_revision` is set to the returned value of `Some(10)`
    /// 4. Checking [`Undo::has_changed_from_revision_index`] with revision `Some(10)` and the
    ///    current data state returns true - the data HAS changed from revision 10, even
    ///    though the data in memory will actually match the data saved to file (since
    ///    both have the same unrecorded changes from step 2).
    ///
    /// In practice this may well be rare - one example would be where very quick data changes
    /// are not recorded while they are in progress (e.g. in a drawing package, while dragging
    /// the mouse to draw a freehand line, where the change is recorded as a new revision only
    /// when the drag completes). In this case, the situation above would only happen if the user
    /// saved the file while in the process of dragging to drawing a line (presumably with
    /// a keyboard shortcut?), and then immediately released the mouse button to complete the
    /// line without drawing any additional segments (so that the data doesn't have any further
    /// changes relative to the file). In this case this would be unlikely to happen in practice.
    ///
    pub fn most_recent_revision_index(&self) -> Option<RevisionIndex> {
        self.undos.back().map(|r| r.index)
    }

    /// True if the data may have changed since the revision with the specified index,
    /// which was usually produced by [`Undo::most_recent_revision_index`]. See that
    /// function for more details on how to use this.
    pub fn may_have_changed_from_revision_index(
        &self,
        index: Option<RevisionIndex>,
        current_data: &D,
    ) -> bool {
        if let (Some(saved_index), Some(current_revision)) = (index, self.undos.back()) {
            if saved_index == current_revision.index {
                // Last undo revision is the same as the saved one -
                // we have changed if the data has changed
                current_data.has_changed_from(&current_revision.data)
            } else {
                // Revision index is different, we've changed
                true
            }
        } else {
            // Either there is no saved index, or no undo revision,
            // we can't  tell whether dat has changed, so default to true
            true
        }
    }

    fn has_changed_from_last_undo(&self, current_data: &D) -> bool {
        if let Some(p) = self.undos.back() {
            current_data.has_changed_from(&p.data)
        } else {
            true
        }
    }

    /// Create a new [`Undo`] with the given [`Settings`].
    pub fn with_settings(settings: Settings) -> Self {
        Self {
            settings,
            ..Default::default()
        }
    }

    /// Do we have an undo point different from the given state?
    pub fn has_undo(&self, current_data: &D) -> bool {
        match self.undos.len() {
            0 => false,
            1 => self.has_changed_from_last_undo(current_data),
            _ => true,
        }
    }

    pub fn has_redo(&self, current_data: &D) -> bool {
        !self.redos.is_empty() && !self.has_changed_from_last_undo(current_data)
    }

    fn new_revision(&mut self, current_data: &D, current_view: &V) -> Revision<D, V> {
        let index = self.next_revision_index;
        self.next_revision_index = self.next_revision_index.next();
        Revision {
            index,
            data: current_data.clone(),
            view: current_view.clone(),
        }
    }

    pub fn undo(&mut self, current_data: &D, current_view: &V) -> Option<(D, V)> {
        if self.has_undo(current_data) {
            // Always use the view from the most recent revision, even though we
            // are usually undoing to the previous revision of the data - this is so
            // that we display the context in which the most recent revision was created,
            // as we undo it. See [`Revision`] docs for more details.
            if let Some(undo_view) = self.undos.back().map(|r| r.view.clone()) {
                if !self.has_changed_from_last_undo(current_data) {
                    #[allow(clippy::unwrap_used)]
                    self.redos.push(self.undos.pop_back().unwrap());
                } else {
                    let new_rev = self.new_revision(current_data, current_view);
                    self.redos.push(new_rev);
                }

                // Note: we keep the undo point intact.
                self.undos.back().map(move |r| (r.data.clone(), undo_view))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn redo(&mut self, current_data: &D) -> Option<(D, V)> {
        if !self.undos.is_empty() && self.has_changed_from_last_undo(current_data) {
            // state changed since the last undo, redos should be cleared.
            self.redos.clear();
            None
        } else if let Some(revision) = self.redos.pop() {
            self.undos.push_back(revision);
            self.undos.back().map(|r| (r.data.clone(), r.view.clone()))
        } else {
            None
        }
    }

    /// Add an undo point if, and only if, there has been a change since the latest undo point.
    pub fn add_undo(&mut self, current_data: &D, current_view: &V) {
        if self.has_changed_from_last_undo(current_data) {
            let new_rev = self.new_revision(current_data, current_view);
            self.undos.push_back(new_rev);
        }
        while self.undos.len() > self.settings.max_undos {
            self.undos.pop_front();
        }
    }

    /// Call this as often as you want (e.g. every frame)
    /// and [`Undoer`] will determine if a new undo point should be created.
    ///
    /// * `current_time`: current time in seconds.
    pub fn feed_state(&mut self, current_data: &D, current_view: &V) {
        match self.undos.back() {
            None => {
                // First time feed_state is called.
                // always create an undo point:
                self.add_undo(current_data, current_view);
            }
            Some(latest_revision) => {
                // Otherwise create an undo only if state has changed
                if current_data.has_changed_from(&latest_revision.data) {
                    self.redos.clear();
                    self.add_undo(current_data, current_view);
                }
            }
        }
    }
}
