use std::fmt::Debug;
use std::hash::Hash;

use egui::ahash::HashMap;
use egui::ahash::HashSet;
use egui::ahash::HashSetExt;

/// A set of items that can be selected, where each item has an id.
/// No two items in the set at the same time can share an id.
/// Allows for updating a [`Selection`] by providing all ids in the list
pub trait Selectable<I> {
    /// The set of ids of all selectable items
    fn all_ids(&self) -> HashSet<I>;

    /// True if the given id is in the set of selectable items
    fn contains_id(&self, id: &I) -> bool;
}

/// A [`Selectable`] that also has an optional default id.
/// Allows for updating an invalid or empty selection with the default.
pub trait SelectableDefault<I>: Selectable<I> {
    /// The default selection id, if there is one
    /// There may be no default, for example if the data is empty
    fn default_id(&self) -> Option<I>;
}

/// A list of items that can be selected, where each item has an id.
/// No two items in the list at the same time can share an id.
/// Allows for updating a [`Selection`] by providing all ids in the list,
/// and the ids that lie between any pair of items.
pub trait SelectableList<I>: Selectable<I> {
    /// Return the ids for all items between item "a" and "b",
    /// where a and b are located using the specified ids.
    /// Items a and b can be in either order, note that the ids between
    /// will always be returned sorted by position in our list of items,
    /// by increasing index.
    /// The returned vec may be empty if one or both of the ids don't match
    /// any item.
    fn id_range(&self, a: I, b: I) -> Vec<I>;
}

/// A [`SelectableList`] that also provides id by index, allowing
/// for additional updates to [`Selection`]
pub trait SelectableListIter<I>: SelectableList<I> {
    /// Get the id at specified index, if there is an item at that index
    fn get_id(&self, i: usize) -> Option<I>;

    /// Get an iterator of ids for the whole list
    fn id_iter(&self) -> impl Iterator<Item = I>;

    /// Get the number of items in the list
    fn id_len(&self) -> usize;
}

/// A [`SelectableListIter`] that also allows for deletion of items by id
pub trait SelectableListIterDeletable<I: Eq + Hash + Copy>: SelectableListIter<I> {
    /// Remove all items with an id in the selection
    /// Return true if any items were removed
    fn delete_by_selection(&mut self, selection: &Selection<I>) -> bool;
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Default, Clone, PartialEq)]
pub enum SelectionState<I> {
    /// No selection actions have occurred yet
    #[default]
    Idle,

    /// The most recent action set an anchor - this is
    /// the starting point of a (potential) range selection
    /// when shift-clicking
    Anchor { anchor: I },

    /// The most recent action was to select a range (via
    /// shift-clicking), and so if we receive another
    /// shift-click, this range should be removed from selection
    /// before adding the new one.
    Range { anchor: I, endpoint: I },
}

/// Tracks a set of selected ids.
/// Note that an item is selected if a) its id is in this selection,
/// and b) an item with that id is present in the data where we're tracking a selection.
/// Therefore when using this, you should be aware that there may be ids
/// for deleted items etc., and ignore these. However, it's still best practice to
/// clear the id for deleted items from a [`Selection`] when deleting.
#[derive(Debug, serde::Deserialize, serde::Serialize, Default, Clone, PartialEq)]
pub struct Selection<I: Eq + Hash + Copy> {
    selected_ids: HashSet<I>,
    pub state: SelectionState<I>,
    /// Override the selected ids, and treat all items as selected
    /// Because this leaves the selected ids in place, it enables toggling
    /// back and forth between selecting all items, and the previous selection,
    /// which is useful since it allows quickly swapping to all layers to e.g.
    /// erase, copy or cut, and then back to a single layer to draw.
    all: bool,
}

/// The directions we can shift a selection
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShiftDirection {
    /// Shift so that lower indices are selected
    DecreaseIndex,

    /// Shift so that higher indices are selected
    IncreaseIndex,
}

impl<I: Eq + Hash + Copy> Selection<I> {
    pub fn new() -> Selection<I> {
        Selection {
            selected_ids: HashSet::new(),
            state: SelectionState::Idle,
            all: false,
        }
    }

    pub fn clear(&mut self) {
        self.selected_ids.clear();
        self.all = false;
        self.state = SelectionState::Idle;
    }

    /// Select only the provided id.
    /// Useful just after adding a new item.
    pub fn select_only(&mut self, id: I) {
        self.clear();
        self.add_id(id);
    }

    /// Try to select only the item with specified index - if there is no such item then
    /// leave selection unchanged.
    /// Return true if selection was changed, false otherwise
    pub fn select_only_index<S: SelectableListIter<I>>(&mut self, index: usize, items: &S) -> bool {
        for (item_index, id) in items.id_iter().enumerate() {
            if item_index == index {
                self.select_only(id);
                return true;
            }
        }
        false
    }

    pub fn is_selected(&self, id: I) -> bool {
        self.all || self.selected_ids.contains(&id)
    }

    /// Toggle the override that selects all items.
    /// When this is true, all items are treated as selected, rather than the underlying
    /// actual selection. Calling this toggle again will then move back to that underlying
    /// selection. Public methods will respect this setting, which may include clearing
    /// the override and updating the underlying selection appropriately.
    pub fn toggle_select_all<S: SelectableList<I>>(&mut self, items: &S) -> bool {
        // Before doing anything else, make sure that any "selected" items that
        // no longer exist are removed from the selection
        let all_ids = items.all_ids();
        self.intersect(&all_ids);

        // We can always toggle self.all off, but only toggle it on if we don't
        // already have everything selected, since this is confusing (it doesn't
        // change the effective selection, but it may lead to losing the anchor
        // when we don't need to)
        if self.all || all_ids.len() != self.selected_ids.len() {
            self.all = !self.all;
        }

        self.all
    }

    pub fn update_from_click<S: SelectableList<I>>(
        &mut self,
        items: &S,
        id: I,
        shift: bool,
        command: bool,
    ) {
        // Before doing anything else, make sure that any "selected" items that
        // no longer exist are removed from the selection
        let all_ids = items.all_ids();
        self.intersect(&all_ids);

        // If all are selected via override, replace selection with all the actual ids,
        // so we can respond appropriately to the click. E.g. if it's a command click we
        // will unselect the clicked item.
        // Note we use clear so that we move to idle state rather than using any state,
        // which would be confusing.
        if self.all {
            self.clear();
            self.selected_ids.extend(all_ids.iter());
        }

        // If our state references an invalid id, revert to idle
        if match self.state {
            SelectionState::Idle => false,
            SelectionState::Anchor { anchor } => !all_ids.contains(&anchor),
            SelectionState::Range { anchor, endpoint } => {
                !all_ids.contains(&anchor) || !all_ids.contains(&endpoint)
            }
        } {
            self.state = SelectionState::Idle;
        }

        // If command is held, this overrides shift, and we just toggle the selected
        // item. If this selects the item (rather than deselecting), record it as
        // the new anchor, otherwise move to idle state (no anchored selection in progress)
        if command {
            self.state = if self.toggle_id(id) {
                SelectionState::Anchor { anchor: id }
            } else {
                SelectionState::Idle
            };

        // If shift is held, but not command, we handle selecting a range from the
        // anchor to new selection. If the previous selection action was to select
        // a range, we first clear that range.
        } else if shift {
            match self.state {
                // No anchor, holding shift is irrelevant, so just treat as a normal
                // click and select the clicked layer, treating it as an anchor
                SelectionState::Idle => {
                    self.state = SelectionState::Anchor { anchor: id };
                    self.add_id(id);
                }

                // We have an anchor but no endpoint, so we just select the range and
                // record the endpoint
                SelectionState::Anchor { anchor } => {
                    self.state = SelectionState::Range {
                        anchor,
                        endpoint: id,
                    };
                    self.add_ids(&items.id_range(anchor, id));
                }

                // Previous action was to select a range, and we are still selecting a range,
                // so we need to clear the old range, select and record the new one
                SelectionState::Range { anchor, endpoint } => {
                    self.remove_ids(&items.id_range(anchor, endpoint));
                    self.state = SelectionState::Range {
                        anchor,
                        endpoint: id,
                    };
                    self.add_ids(&items.id_range(anchor, id));
                }
            }

        // No modifier held, so we clear the selection, add the one clicked, and make it the new anchor
        } else {
            self.clear();
            self.state = SelectionState::Anchor { anchor: id };
            self.add_id(id);
        }
    }

    /// Find the indices of the selected ids, in the specified items
    /// So for example if the items have ids [42, 3, 12], and we have
    /// ids 3 and 12 selected, then the indices would be [1, 2], since
    /// id 3 is at index 1, and id 12 is at index 2.
    pub fn selected_indices<S: SelectableListIter<I>>(&mut self, items: &S) -> Vec<usize> {
        items
            .id_iter()
            .enumerate()
            .filter(|(_index, id)| self.is_selected(*id))
            .map(|(index, _id)| index)
            .collect()
    }

    pub fn can_shift_selection<S: SelectableListIter<I>>(
        &mut self,
        items: &S,
        direction: ShiftDirection,
    ) -> bool {
        let selected_indices = self.selected_indices(items);
        self.can_shift_selection_from_indices(items, &selected_indices, direction)
    }

    /// This just moves the layer selection - not the layers themselves,
    /// just the pattern of which layers are selected.
    /// If higher is true, for each selected layer, the layer above it
    /// is selected instead. If higher is false, the layers underneath are selected.
    pub fn shift_selection<S: SelectableListIter<I>>(
        &mut self,
        items: &S,
        direction: ShiftDirection,
    ) {
        let selected_indices = self.selected_indices(items);

        if self.can_shift_selection_from_indices(items, &selected_indices, direction) {
            self.clear();
            for i in selected_indices.iter() {
                let new_i = match direction {
                    ShiftDirection::DecreaseIndex => i - 1,
                    ShiftDirection::IncreaseIndex => i + 1,
                };
                if let Some(id) = items.get_id(new_i) {
                    self.add_id(id);
                }
            }
        }
    }

    /// Delete the selected items from provided list, and update selection accordingly
    pub fn delete_selected_items<S: SelectableListIterDeletable<I>>(
        &mut self,
        items: &mut S,
    ) -> bool {
        // Work out which id to select after deleting selected items.
        // We use the last item to work out what to do - if there is no last item
        // then there's nothing to delete
        let selected_id_after_deletion = if let Some(last_id) = items.id_iter().last() {
            // If last item will be deleted, then we will select the last
            // unselected item (i.e. the last that won't be deleted), if any exists.
            if self.is_selected(last_id) {
                let mut last_unselected_id: Option<I> = None;
                for id in items.id_iter() {
                    if !self.is_selected(id) {
                        last_unselected_id = Some(id);
                    }
                }
                last_unselected_id

            // The last item isn't going to be deleted, therefore
            // we can find the last item that is going to be deleted,
            // and select the item after it (since we know such an
            // item exists - if nothing else it will be the last item)
            } else {
                let mut last_deleted_index = None;
                for (index, id) in items.id_iter().enumerate() {
                    if self.is_selected(id) {
                        last_deleted_index = Some(index);
                    }
                }
                last_deleted_index.and_then(|i| items.get_id(i + 1))
            }
        } else {
            // There are no items, might as well clear selection in case there are dangling ids
            self.clear();
            None
        };

        // Perform the deletion, and if it changes anything, update the selection
        let change = items.delete_by_selection(self);
        if change {
            self.clear();
            if let Some(id) = selected_id_after_deletion {
                self.add_id(id);
            }
        }

        change
    }

    fn can_shift_selection_from_indices<S: SelectableListIter<I>>(
        &mut self,
        items: &S,
        selected_indices: &[usize],
        direction: ShiftDirection,
    ) -> bool {
        match direction {
            ShiftDirection::DecreaseIndex => {
                selected_indices.first().map(|i| *i > 0).unwrap_or(false)
            }
            ShiftDirection::IncreaseIndex => {
                let id_len = items.id_len();
                selected_indices
                    .last()
                    .map(|i| *i < id_len - 1)
                    .unwrap_or(false)
            }
        }
    }

    fn add_id(&mut self, id: I) {
        self.selected_ids.insert(id);
    }

    fn remove_id(&mut self, id: &I) {
        self.selected_ids.remove(id);
    }

    fn add_ids(&mut self, ids: &[I]) {
        for id in ids.iter() {
            self.add_id(*id);
        }
    }

    fn remove_ids(&mut self, ids: &[I]) {
        for id in ids.iter() {
            self.remove_id(id);
        }
    }

    fn toggle_id(&mut self, id: I) -> bool {
        if !self.selected_ids.contains(&id) {
            self.selected_ids.insert(id);
            true
        } else {
            self.selected_ids.remove(&id);
            false
        }
    }

    fn intersect(&mut self, ids: &HashSet<I>) {
        self.selected_ids.retain(|id| ids.contains(id));
    }
}

/// If `selection` is [`None`], or is `Some(id)` where `id` is not in the
/// [`Selectable`], then set it to the default selection for the [`Selectable`]
pub fn apply_default_selection<I, T: SelectableDefault<I>>(
    selection: &mut Option<I>,
    selectable: &T,
) {
    if !selection
        .as_ref()
        .map(|id| selectable.contains_id(id))
        .unwrap_or(false)
    {
        *selection = selectable.default_id();
    }
}

/// Ensure that `values_by_id` contains exactly one mapping
/// for each id in the [`Selectable`]. Where there are values from ids
/// not present in the [`Selectable`], they are removed. Where there is
/// no value for an id in [`Selectable`], a new value is added using the
/// default value of `S` (this is often a selection, but can be any type
/// with a [`Default`]).
pub fn apply_default_value_per_selectable_id<I: Eq + Hash, T: Selectable<I>, S: Default>(
    values_by_id: &mut HashMap<I, S>,
    selectable: &T,
) {
    let all_ids = selectable.all_ids();

    // Remove selections for any ids not in current tilesets
    values_by_id.retain(|id, _sel| all_ids.contains(id));

    // Add empty selections for any tilesets that don't have one
    for id in all_ids {
        values_by_id.entry(id).or_default();
    }
}
