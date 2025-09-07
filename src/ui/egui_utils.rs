use egui::{
    text::{CCursor, CCursorRange},
    InnerResponse, Layout, Response, RichText, Ui, Vec2, WidgetText,
};

use crate::{
    data::tiles::tile_color::UserColor,
    ui::{
        color_edit::color_edit_button, theme::DEFAULT_THEME,
        tileset_image_loader::TilesetImageLoader,
    },
};

pub fn replace_fonts(ctx: &egui::Context) {
    // Start with the default fonts (we will be adding to them rather than replacing them).
    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "InterNerdFont-Regular".to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
            "../../assets/InterNerdFont-Regular.ttf"
        ))),
    );

    // Put Inter first (highest priority) for proportional text:
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "InterNerdFont-Regular".to_owned());

    // Tell egui to use these fonts:
    ctx.set_fonts(fonts);
}

/// Add a single line text edit to ui, optionally requesting focus and selecting all text
/// contents. In most cases you will want to use some state to make sure that `init` is true
/// only when the component is first displayed - e.g. see how [`crate::state::ModalResult`] is
/// used for modals.
pub fn singleline_focus_and_select(ui: &mut Ui, text: &mut String, init: bool) -> Response {
    let mut output = egui::TextEdit::singleline(text).show(ui);
    // See:
    // https://stackoverflow.com/questions/74324236/select-the-text-of-a-textedit-object-in-egui
    // https://docs.rs/egui/latest/egui/widgets/text_edit/struct.TextEdit.html#method.show
    if init {
        output.response.request_focus();

        output.state.cursor.set_char_range(Some(CCursorRange::two(
            CCursor::new(0),
            CCursor::new(text.len()),
        )));

        // don't forget to do apply changes
        output.state.store(ui.ctx(), output.response.id)
    }
    output.response
}

pub fn sized_button(ui: &mut Ui, size: impl Into<Vec2>, text: impl Into<WidgetText>) -> Response {
    ui.add_sized(size, egui::Button::new(text))
}

pub fn square_button(ui: &mut Ui, text: impl Into<WidgetText>) -> Response {
    sized_button(
        ui,
        [DEFAULT_THEME.control_height, DEFAULT_THEME.control_height],
        text,
    )
}

pub fn enabled_square_button(ui: &mut Ui, enabled: bool, text: impl Into<WidgetText>) -> Response {
    ui.add_enabled_ui(enabled, |ui| square_button(ui, text))
        .inner
}

pub fn unselectable_label(ui: &mut Ui, text: impl Into<WidgetText>) -> Response {
    ui.add(egui::Label::new(text).selectable(false))
}

pub fn unselectable_label_strong(ui: &mut Ui, text: impl Into<String>) -> Response {
    ui.add(egui::Label::new(RichText::new(text).strong()).selectable(false))
}

pub fn separator(ui: &mut Ui) -> Response {
    ui.add(egui::Separator::default().spacing(DEFAULT_THEME.separator_spacing))
}

/// See [`egui::ui::Ui::add_sized`] - this is similar but using main layout dir
/// with cross_align center,and accepting a closure rather than an [`egui::Widget`],
/// since this seems to be what I always want...
pub fn sized_main_dir_cross_align_center<R>(
    ui: &mut Ui,
    max_size: impl Into<Vec2>,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<R> {
    let layout = Layout::from_main_dir_and_cross_align(ui.layout().main_dir(), egui::Align::Center);
    ui.allocate_ui_with_layout(max_size.into(), layout, add_contents)
}

pub fn user_color_edit_button(
    ui: &mut Ui,
    color: &mut UserColor,
    as_text: &mut String,
) -> Response {
    // ui.color_edit_button_srgba_unmultiplied(color.slice_mut())
    color_edit_button(ui, color, as_text)
}

pub fn install_image_loaders(ctx: &egui::Context) {
    if !ctx.is_loader_installed(TilesetImageLoader::ID) {
        ctx.add_image_loader(std::sync::Arc::new(TilesetImageLoader::default()));
        log::trace!("installed TilesetImageLoader");
    }
}
