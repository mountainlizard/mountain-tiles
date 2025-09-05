use egui::{Context, Key, KeyboardShortcut, Modifiers};

use crate::{
    app::App, geom::i32pos2::i32pos2, geom::transform::Transform, selection::ShiftDirection,
};

const NEW_SHORTCUT: KeyboardShortcut = KeyboardShortcut {
    modifiers: Modifiers::COMMAND,
    logical_key: Key::N,
};
const OPEN_SHORTCUT: KeyboardShortcut = KeyboardShortcut {
    modifiers: Modifiers::COMMAND,
    logical_key: Key::O,
};
const SAVE_AS_SHORTCUT: KeyboardShortcut = KeyboardShortcut {
    modifiers: Modifiers {
        alt: false,
        ctrl: false,
        shift: true,
        mac_cmd: false,
        command: true,
    },
    logical_key: Key::S,
};
const SAVE_SHORTCUT: KeyboardShortcut = KeyboardShortcut {
    modifiers: Modifiers::COMMAND,
    logical_key: Key::S,
};

const EXPORT_PNG_SHORTCUT: KeyboardShortcut = KeyboardShortcut {
    modifiers: Modifiers::COMMAND,
    logical_key: Key::E,
};

const RESET_ZOOM_SHORTCUT: KeyboardShortcut = KeyboardShortcut {
    modifiers: Modifiers::COMMAND,
    logical_key: Key::R,
};

const REDO_SHORTCUT: KeyboardShortcut = KeyboardShortcut {
    modifiers: Modifiers::COMMAND,
    logical_key: Key::Y,
};
const REDO_SHORTCUT_ALT: KeyboardShortcut = KeyboardShortcut {
    modifiers: Modifiers {
        shift: true,
        command: true,
        alt: false,
        ctrl: false,
        mac_cmd: false,
    },
    logical_key: Key::Z,
};
const UNDO_SHORTCUT: KeyboardShortcut = KeyboardShortcut {
    modifiers: Modifiers::COMMAND,
    logical_key: Key::Z,
};

// We use non-standard shortcuts for copy/paste on map
// This is because the normal cmd+x/c/v shortcuts are grabbed
// by egui so we never see them. This might actually be fine
// long-term, since we're not using the system clipboard, but
// it could also be fixed by forking and using something like:
// https://github.com/emilk/egui/pull/5615
// This is linked from the main issue:
// https://github.com/emilk/egui/issues/4065
const COPY_SHORTCUT: KeyboardShortcut = KeyboardShortcut {
    modifiers: Modifiers::NONE,
    logical_key: Key::C,
};
const CUT_SHORTCUT: KeyboardShortcut = KeyboardShortcut {
    modifiers: Modifiers::NONE,
    logical_key: Key::X,
};
const PASTE_SHORTCUT: KeyboardShortcut = KeyboardShortcut {
    modifiers: Modifiers::NONE,
    logical_key: Key::V,
};
const PASTE_SHORTCUT_ALT: KeyboardShortcut = KeyboardShortcut {
    modifiers: Modifiers::NONE,
    logical_key: Key::P,
};
const ROTATE_SHORTCUT: KeyboardShortcut = KeyboardShortcut {
    modifiers: Modifiers::NONE,
    logical_key: Key::R,
};
const ROTATE_SHORTCUT_MAC: KeyboardShortcut = KeyboardShortcut {
    modifiers: Modifiers::NONE,
    logical_key: Key::Z,
};
const MIRROR_X_SHORTCUT: KeyboardShortcut = KeyboardShortcut {
    modifiers: Modifiers::NONE,
    logical_key: Key::T,
};
const MIRROR_Y_SHORTCUT: KeyboardShortcut = KeyboardShortcut {
    modifiers: Modifiers::NONE,
    logical_key: Key::Y,
};
const CLEAR_TRANSFORM_SHORTCUT: KeyboardShortcut = KeyboardShortcut {
    modifiers: Modifiers::NONE,
    logical_key: Key::U,
};
const PREVIOUS_TILESET_SHORTCUT: KeyboardShortcut = KeyboardShortcut {
    modifiers: Modifiers::NONE,
    logical_key: Key::Minus,
};
const NEXT_TILESET_SHORTCUT: KeyboardShortcut = KeyboardShortcut {
    modifiers: Modifiers::NONE,
    logical_key: Key::Equals,
};
const PREVIOUS_PALETTE_INDEX_SHORTCUT: KeyboardShortcut = KeyboardShortcut {
    modifiers: Modifiers::NONE,
    logical_key: Key::OpenBracket,
};
const NEXT_PALETTE_INDEX_SHORTCUT: KeyboardShortcut = KeyboardShortcut {
    modifiers: Modifiers::NONE,
    logical_key: Key::CloseBracket,
};
const PREVIOUS_LAYER_SHORTCUT: KeyboardShortcut = KeyboardShortcut {
    modifiers: Modifiers::NONE,
    logical_key: Key::Comma,
};
const NEXT_LAYER_SHORTCUT: KeyboardShortcut = KeyboardShortcut {
    modifiers: Modifiers::NONE,
    logical_key: Key::Period,
};
const PREVIOUS_LAYER_SHORTCUT_ALT: KeyboardShortcut = KeyboardShortcut {
    modifiers: Modifiers::NONE,
    logical_key: Key::Quote,
};
const NEXT_LAYER_SHORTCUT_ALT: KeyboardShortcut = KeyboardShortcut {
    modifiers: Modifiers::NONE,
    logical_key: Key::Backslash,
};

const DRAW_MODE_SHORTCUT: KeyboardShortcut = KeyboardShortcut {
    modifiers: Modifiers::NONE,
    logical_key: Key::D,
};
const SELECT_MODE_SHORTCUT: KeyboardShortcut = KeyboardShortcut {
    modifiers: Modifiers::NONE,
    logical_key: Key::S,
};
const ERASE_MODE_SHORTCUT: KeyboardShortcut = KeyboardShortcut {
    modifiers: Modifiers::NONE,
    logical_key: Key::E,
};
const DELETE_SHORTCUT: KeyboardShortcut = KeyboardShortcut {
    modifiers: Modifiers::NONE,
    logical_key: Key::Delete,
};
const BACKSPACE_SHORTCUT: KeyboardShortcut = KeyboardShortcut {
    modifiers: Modifiers::NONE,
    logical_key: Key::Backspace,
};

const TOGGLE_SELECT_ALL_LAYERS_SHORTCUT: KeyboardShortcut = KeyboardShortcut {
    modifiers: Modifiers::NONE,
    logical_key: Key::A,
};

const LAYER_SHORTCUTS: [KeyboardShortcut; 9] = [
    KeyboardShortcut {
        modifiers: Modifiers::NONE,
        logical_key: Key::Num1,
    },
    KeyboardShortcut {
        modifiers: Modifiers::NONE,
        logical_key: Key::Num2,
    },
    KeyboardShortcut {
        modifiers: Modifiers::NONE,
        logical_key: Key::Num3,
    },
    KeyboardShortcut {
        modifiers: Modifiers::NONE,
        logical_key: Key::Num4,
    },
    KeyboardShortcut {
        modifiers: Modifiers::NONE,
        logical_key: Key::Num5,
    },
    KeyboardShortcut {
        modifiers: Modifiers::NONE,
        logical_key: Key::Num6,
    },
    KeyboardShortcut {
        modifiers: Modifiers::NONE,
        logical_key: Key::Num7,
    },
    KeyboardShortcut {
        modifiers: Modifiers::NONE,
        logical_key: Key::Num8,
    },
    KeyboardShortcut {
        modifiers: Modifiers::NONE,
        logical_key: Key::Num9,
    },
];

const TILESET_LEFT_SHORTCUT: KeyboardShortcut = KeyboardShortcut {
    modifiers: Modifiers::NONE,
    logical_key: Key::ArrowLeft,
};
const TILESET_RIGHT_SHORTCUT: KeyboardShortcut = KeyboardShortcut {
    modifiers: Modifiers::NONE,
    logical_key: Key::ArrowRight,
};
const TILESET_UP_SHORTCUT: KeyboardShortcut = KeyboardShortcut {
    modifiers: Modifiers::NONE,
    logical_key: Key::ArrowUp,
};
const TILESET_DOWN_SHORTCUT: KeyboardShortcut = KeyboardShortcut {
    modifiers: Modifiers::NONE,
    logical_key: Key::ArrowDown,
};

const HELP_SHORTCUT: KeyboardShortcut = KeyboardShortcut {
    modifiers: Modifiers::NONE,
    logical_key: Key::H,
};

pub fn consume_shortcuts(ctx: &Context, app: &mut App) {
    ctx.input_mut(|i| {
        if i.consume_shortcut(&NEW_SHORTCUT) {
            app.check_data_loss_then_new_document();
        }
        if i.consume_shortcut(&OPEN_SHORTCUT) {
            app.check_data_loss_then_show_open_document_modal();
        }
        if i.consume_shortcut(&SAVE_AS_SHORTCUT) {
            app.show_save_as_document_modal();
        }
        if i.consume_shortcut(&SAVE_SHORTCUT) {
            app.show_save_document_modal();
        }

        if i.consume_shortcut(&EXPORT_PNG_SHORTCUT) {
            app.show_export_png_modal();
        }

        if i.consume_shortcut(&RESET_ZOOM_SHORTCUT) {
            app.reset_selected_map_zoom();
        }

        if i.consume_shortcut(&CUT_SHORTCUT) {
            app.cut();
        }
        if i.consume_shortcut(&COPY_SHORTCUT) {
            app.copy();
        }
        if i.consume_shortcut(&PASTE_SHORTCUT) || i.consume_shortcut(&PASTE_SHORTCUT_ALT) {
            app.draw_mode();
        }
        if i.consume_shortcut(&DRAW_MODE_SHORTCUT) {
            app.draw_mode();
        }
        if i.consume_shortcut(&REDO_SHORTCUT_ALT) {
            app.redo();
        };
        if i.consume_shortcut(&REDO_SHORTCUT) {
            app.redo();
        };
        if i.consume_shortcut(&UNDO_SHORTCUT) {
            app.undo();
        };
        if i.consume_shortcut(&ROTATE_SHORTCUT) || i.consume_shortcut(&ROTATE_SHORTCUT_MAC) {
            app.transform(Transform::Rotate90);
        }
        if i.consume_shortcut(&MIRROR_X_SHORTCUT) {
            app.transform(Transform::MirrorX);
        }
        if i.consume_shortcut(&MIRROR_Y_SHORTCUT) {
            app.transform(Transform::MirrorXRotate180);
        }
        if i.consume_shortcut(&CLEAR_TRANSFORM_SHORTCUT) {
            app.clear_transform();
        }

        if i.consume_shortcut(&PREVIOUS_TILESET_SHORTCUT) {
            app.previous_tileset();
        }
        if i.consume_shortcut(&NEXT_TILESET_SHORTCUT) {
            app.next_tileset();
        }

        if i.consume_shortcut(&PREVIOUS_PALETTE_INDEX_SHORTCUT) {
            app.previous_palette_index();
        }
        if i.consume_shortcut(&NEXT_PALETTE_INDEX_SHORTCUT) {
            app.next_palette_index();
        }

        if i.consume_shortcut(&PREVIOUS_LAYER_SHORTCUT)
            || i.consume_shortcut(&PREVIOUS_LAYER_SHORTCUT_ALT)
        {
            app.shift_layer_selection(ShiftDirection::DecreaseIndex);
        }
        if i.consume_shortcut(&NEXT_LAYER_SHORTCUT) || i.consume_shortcut(&NEXT_LAYER_SHORTCUT_ALT)
        {
            app.shift_layer_selection(ShiftDirection::IncreaseIndex);
        }

        if i.consume_shortcut(&SELECT_MODE_SHORTCUT) {
            app.select_mode();
        }
        if i.consume_shortcut(&ERASE_MODE_SHORTCUT) {
            app.erase_mode();
        }
        if i.consume_shortcut(&DELETE_SHORTCUT) || i.consume_shortcut(&BACKSPACE_SHORTCUT) {
            app.delete_and_clear_selection();
        }

        if i.consume_shortcut(&TOGGLE_SELECT_ALL_LAYERS_SHORTCUT) {
            app.selected_map_toggle_select_all_layers();
        }

        for (layer_index, layer_shortcut) in LAYER_SHORTCUTS.iter().enumerate() {
            if i.consume_shortcut(layer_shortcut) {
                app.select_layer(layer_index)
            }
        }

        if i.consume_shortcut(&TILESET_UP_SHORTCUT) {
            app.shift_tileset_selection(i32pos2(0, -1));
        }
        if i.consume_shortcut(&TILESET_DOWN_SHORTCUT) {
            app.shift_tileset_selection(i32pos2(0, 1));
        }
        if i.consume_shortcut(&TILESET_LEFT_SHORTCUT) {
            app.shift_tileset_selection(i32pos2(-1, 0));
        }
        if i.consume_shortcut(&TILESET_RIGHT_SHORTCUT) {
            app.shift_tileset_selection(i32pos2(1, 0));
        }
        if i.consume_shortcut(&HELP_SHORTCUT) {
            app.show_help_modal();
        }
    });
}
