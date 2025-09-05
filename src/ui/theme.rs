use egui::containers::Frame;
use egui::style::{Selection, Widgets};
use egui::{Color32, CornerRadius, Margin, Shadow, Stroke};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Theme {
    pub base50: Color32,
    pub base100: Color32,
    pub base200: Color32,
    pub base300: Color32,
    pub base_content: Color32,
    pub base_subcontent: Color32,
    pub selected: Color32,
    pub selected_content: Color32,
    pub selected_border: Color32,
    pub selected_fill: Color32,
    pub selected_tile: Color32,
    pub unselected_tile: Color32,
    pub erase_fill: Color32,
    pub inactive_widget_bg: Color32,
    pub hovered_widget_bg: Color32,
    pub active_widget_bg: Color32,
    pub modal_spacing: f32,
    pub separator_spacing: f32,
    pub control_height: f32,
    pub row_height: f32,
    pub selected_stroke: Stroke,
}

impl Theme {
    pub fn frame(fill: Color32, margin: i8) -> Frame {
        egui::containers::Frame {
            inner_margin: Margin::same(margin),
            stroke: Stroke::NONE,
            fill,
            corner_radius: CornerRadius::ZERO,
            outer_margin: Margin::ZERO,
            shadow: Shadow::NONE,
        }
    }

    // pub fn base_50_frame(&self, margin: i8) -> Frame {
    //     Self::frame(self.base50, margin)
    // }
    pub fn base_100_frame(&self, margin: i8) -> Frame {
        Self::frame(self.base100, margin)
    }
    pub fn base_200_frame(&self, margin: i8) -> Frame {
        Self::frame(self.base200, margin)
    }
    pub fn modal_frame(&self) -> Frame {
        self.base_100_frame(16)
    }
    // pub fn base_300_frame(&self, margin: i8) -> Frame {
    //     Self::frame(self.base300, margin)
    // }
}

pub const DEFAULT_THEME: Theme = Theme {
    // Note - these are taken from mountain configurator theme in
    // lightbox-picow-rs, converted using https://oklch.com
    // base50 is just a brighter and slightly higher chroma version of base100
    // The tailwind colors are here - note we set up the "info" color for selection color:
    // https://daisyui.com/theme-generator/#theme=eJyVlO2OoyAUhm-FmGyyk7QEDoKwd0P1uDVjpQGb-crc-1BsWqjuj_0J53l5z5d-VZM9YfWn6qx_rXZV60bn96E9Yn673y_3BxtwzxmLEfc6tsffgv8ijDJQioCEl2cUMhTqhPKm2URFhnKVUGa20dZNM07zHTdiwaUmoGSOn_1wsv7jTuqa6vQ2lxqIFlTzDXxlwOVSZUN0nfMBI9kVBoxKSLDQdcxGUw2birWHoIrdpDwqTS6zbZuzMXHZpDJqpgg3sao1vXaoqVlmADKKikZNeJm9HbNhUQ7ZvCQtE7rx_zGKYerdHavNFYv9BANUlkO4guvcGYsKwopeXmKdITy6omKzU1cUrwkHQ2ED_9dwxVrxZv00TH8fBoaqxYDLuD2KNmIDXxsoqh4LFFVFX9B75_MFhaWFjWwI17TRK3hrsGL5YkSs-zYnb7vhEuLCjdjOySGupsdTHuwHHLsUgefQwb2XkjB8YvkalLGtxw7Od3il-fk9XXR4no9XKJ0mNwS8nTrs7WWMRfV2DLirzh579CH9gJa77x-xNVFr

    // oklch(0.38 0.0346 252)
    base50: Color32::from_rgb(53, 68, 85),
    // oklch(31% 0.0266 252)
    base100: Color32::from_rgb(39, 49, 62),
    // oklch(24% 0.0177 252)
    base200: Color32::from_rgb(25, 32, 40),
    // oklch(16% 0.0097 252)
    base300: Color32::from_rgb(10, 14, 17),
    // oklch(93% 0.0058 265)
    base_content: Color32::from_rgb(230, 232, 236),

    // oklch(0.75 0.0058 265)
    base_subcontent: Color32::from_rgb(172, 174, 178),

    // Just violet-600 from tailwind theme generator
    // oklch(54% 0.281 293.009)
    selected: Color32::from_rgb(127, 33, 254),
    // selected_border: Color32::from_rgb(127, 33, 254),
    selected_border: Color32::from_rgb(196, 179, 255),
    // selected_fill: Color32::from_rgba_premultiplied(96, 24, 192, 100),
    selected_fill: Color32::from_rgba_premultiplied(96, 24, 192, 120),
    // White for slightly higher contrast where used
    selected_content: Color32::from_rgb(255, 255, 255),

    // Adjusted versions of base color and primary
    selected_tile: Color32::from_rgb(254, 210, 65),
    unselected_tile: Color32::from_rgb(79, 83, 92),

    erase_fill: Color32::from_rgba_premultiplied(192, 24, 24, 50),

    // These are taken from the tailwind theme generator, for the "Draft" button, unhovered and hovered
    inactive_widget_bg: Color32::from_rgb(26, 32, 40),
    hovered_widget_bg: Color32::from_rgb(22, 28, 34),
    active_widget_bg: Color32::from_rgb(31, 38, 46),

    modal_spacing: 12.0,

    separator_spacing: 16.0,

    control_height: 24.0,

    row_height: 24.0,
    selected_stroke: Stroke {
        width: 1.0,
        color: Color32::from_rgb(230, 232, 236),
    },
};

pub fn apply_theme(ctx: &egui::Context, colors: Theme) {
    let mut widgets = Widgets::dark().clone();

    // The style of an interactive widget, such as a button, at rest.
    widgets.inactive.bg_fill = colors.inactive_widget_bg;
    widgets.inactive.weak_bg_fill = colors.inactive_widget_bg;

    // The style of an interactive widget, such as a button, at rest.
    widgets.hovered.bg_fill = colors.hovered_widget_bg;
    widgets.hovered.weak_bg_fill = colors.hovered_widget_bg;
    widgets.hovered.bg_stroke = widgets.inactive.bg_stroke;

    // The style of an interactive widget as you are clicking or dragging it.
    widgets.active.bg_fill = colors.active_widget_bg;
    widgets.active.weak_bg_fill = colors.active_widget_bg;
    // widgets.active.bg_stroke = widgets.inactive.bg_stroke;

    widgets.noninteractive.bg_stroke = Stroke::new(1.0, colors.base50);

    ctx.set_visuals_of(
        egui::Theme::Dark,
        egui::Visuals {
            window_fill: colors.base100,
            window_corner_radius: CornerRadius::ZERO,
            menu_corner_radius: CornerRadius::ZERO,
            panel_fill: colors.base100,
            popup_shadow: Shadow::NONE,
            window_shadow: Shadow::NONE,
            override_text_color: Some(colors.base_content),
            widgets,
            selection: Selection {
                bg_fill: colors.selected,
                stroke: Stroke::new(1.0, colors.selected_content),
            },

            // bg_fill: Color32::from_rgb(0, 92, 128),
            // stroke: Stroke::new(1.0, Color32::from_rgb(192, 222, 255)),
            ..Default::default()
        },
    );

    // ctx.set_visuals_of(
    //     egui::Theme::Light,
    //     egui::Visuals {
    //         panel_fill: egui::Color32::GREEN,
    //         ..Default::default()
    //     },
    // );

    // ctx.set_visuals(theme.visuals(ctx.style().visuals.clone()));
}
