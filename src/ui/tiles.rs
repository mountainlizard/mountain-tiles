use egui::{
    load::{SizedTexture, TexturePoll},
    pos2, vec2, Color32, Context, Mesh, Pos2, Rect, Sense, Shape, Vec2,
};

use crate::{
    data::palette::Palette,
    data::tiles::{
        tile_selection::{SelectionType, TileSelection},
        Tiles,
    },
    data::tilesets::{Tileset, Tilesets},
    geom::transform::Transform,
    geom::u32pos2::u32pos2,
    ui::theme::DEFAULT_THEME,
    ui::tile_mesh::TileMesh,
    ui::tileset_textures::{ErrorTexture, TilesetTextures},
};

const SELECTION_FILL_COLOR: Color32 = DEFAULT_THEME.selected_fill;
// const SELECTION_DRAG_COLOR: Color32 = DEFAULT_THEME.selected_drag;
const ERASE_FILL_COLOR: Color32 = DEFAULT_THEME.erase_fill;
// const SELECTION_DRAG_STROKE: Stroke = Stroke {
//     width: 1.0,
//     color: DEFAULT_THEME.selected_border,
// };

#[derive(Debug, Clone, Copy)]
pub enum Overlay {
    None,
    Checkerboard { color: Color32 },
}

fn load_texture(
    textures: &TilesetTextures,
    ctx: &Context,
    tileset: &Tileset,
) -> Option<(SizedTexture, bool)> {
    match textures.texture_for_tileset(ctx, tileset) {
        Ok(TexturePoll::Ready { texture }) => Some((texture, true)),
        Ok(TexturePoll::Pending { .. }) => None,
        Err(_) => match textures.error_texture(ctx, ErrorTexture::MissingImage) {
            Ok(TexturePoll::Ready { texture }) => Some((texture, false)),
            _ => None,
        },
    }
}

#[allow(clippy::too_many_arguments)]
fn tiles_ui<T: Tiles>(
    ui: &mut egui::Ui,
    tiles: &T,
    palette: &Palette,
    tilesets: &Tilesets,
    textures: &TilesetTextures,
    selection: Option<&TileSelection>,
    overlay: Overlay,
    sense: Sense,
) -> egui::Response {
    let desired_size = tiles.pixel_size();

    // let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    let (rect, response) = ui.allocate_exact_size(desired_size, sense);

    if ui.is_rect_visible(rect) {
        let tile_and_gap_size = tiles.tile_size() + tiles.gap();

        let mut half_gap_size: Vec2 = tiles.gap().into();
        half_gap_size /= 2.0;

        let ctx = ui.ctx();
        ui.painter()
            .rect_filled(rect, 0.0, tiles.background().as_premultiplied_color32());
        // Note that layers are drawn in reverse - the first layer is "on top" (and so is drawn over other layers),
        // the last layer is "at the bottom".
        // This is done so that the order of layers in the data vec and list UI are as expected by the user, with earlier
        // layers higher in the "stack", and we only need to reverse the vec here when drawing.
        for layer_index in (0..tiles.layer_count()).rev() {
            let opacity = tiles.layer_opacity(layer_index);
            for tileset in tilesets.iter() {
                if let Some((texture, success)) = load_texture(textures, ctx, tileset) {
                    let mut mesh = Mesh::with_texture(texture.id);
                    let tile_set_size = tileset.size_in_tiles;
                    let tile_set_size_f: Vec2 = tile_set_size.into();
                    let tile_uv_size = vec2(1.0, 1.0) / tile_set_size_f;

                    for grid_pos in tiles.map_positions() {
                        let screen_pos = (Pos2::from(grid_pos * tile_and_gap_size) + half_gap_size)
                            * tiles.scale()
                            + rect.min.to_vec2();
                        let screen_size = Vec2::from(tiles.tile_size()) * tiles.scale();

                        if let Some(ref tile) = tiles.tile(layer_index, grid_pos) {
                            if tile.source.tileset_id == tileset.id() {
                                let tile_uv_pos = (Vec2::from(
                                    tile_set_size
                                        .pos_from_linear_index(tile.source.tile_index.index()),
                                ) / tile_set_size_f)
                                    .to_pos2();
                                let uv = if success {
                                    Rect::from_min_size(tile_uv_pos, tile_uv_size)
                                } else {
                                    Rect::from_min_size(pos2(0.0, 0.0), vec2(1.0, 1.0))
                                };
                                let transform = if success {
                                    tile.transform
                                } else {
                                    Transform::None
                                };

                                let color = match opacity {
                                    Some(opacity) => tile
                                        .color
                                        .as_foreground_user_color(palette)
                                        .with_opacity(opacity)
                                        .as_premultiplied_color32(),
                                    None => tile.color.as_premultiplied_foreground_color32(palette),
                                };

                                mesh.add_rect_with_transform(
                                    Rect::from_min_size(screen_pos, screen_size),
                                    uv,
                                    transform,
                                    color,
                                );
                            }
                        }
                    }
                    ui.painter().add(Shape::mesh(mesh));
                }
            }
        }

        if let Some(selection) = selection {
            // Draw selection
            let selection_fill_color = match selection.selection_type() {
                SelectionType::Selection => SELECTION_FILL_COLOR,
                SelectionType::Erase => ERASE_FILL_COLOR,
            };
            for pos in selection.iter() {
                let screen_pos =
                    Pos2::from(*pos * tile_and_gap_size) * tiles.scale() + rect.min.to_vec2();
                let screen_size = Vec2::from(tile_and_gap_size) * tiles.scale();
                ui.painter().rect_filled(
                    Rect::from_min_size(screen_pos, screen_size),
                    0.0,
                    selection_fill_color,
                );
            }

            // Draw selection drag box
            if let Some(drag_rect) = selection.drag_rect {
                let pos_rect = drag_rect.with_positive_size();
                let screen_pos = Pos2::from(pos_rect.min * tile_and_gap_size) * tiles.scale()
                    + rect.min.to_vec2();
                let screen_size = Vec2::from(tile_and_gap_size * pos_rect.size()) * tiles.scale();
                // ui.painter().rect_stroke(
                //     Rect::from_min_size(screen_pos, screen_size),
                //     0.0,
                //     SELECTION_DRAG_STROKE,
                //     StrokeKind::Inside,
                // );
                ui.painter().rect_filled(
                    Rect::from_min_size(screen_pos, screen_size),
                    0.0,
                    selection_fill_color,
                );
            }
        }

        // Draw overlay
        if let Overlay::Checkerboard { color } = overlay {
            for y in 0..tiles.map_size().h {
                for x in 0..tiles.map_size().w {
                    if (x + y) % 2 == 0 {
                        let pos = u32pos2(x, y);
                        let screen_pos = Pos2::from(pos * tile_and_gap_size) * tiles.scale()
                            + rect.min.to_vec2();
                        let screen_size = Vec2::from(tile_and_gap_size) * tiles.scale();
                        ui.painter().rect_filled(
                            Rect::from_min_size(screen_pos, screen_size),
                            0.0,
                            color,
                        );
                    }
                }
            }
        }
    }

    response
}

/// Tile map
pub fn tiles<'a, T: Tiles>(
    tiles: &'a T,
    palette: &'a Palette,
    tilesets: &'a Tilesets,
    textures: &'a TilesetTextures,
    selection: Option<&'a TileSelection>,
    overlay: Overlay,
) -> impl egui::Widget + 'a {
    tiles_with_sense(
        tiles,
        palette,
        tilesets,
        textures,
        selection,
        overlay,
        egui::Sense::drag(),
    )
}

/// Tile map
pub fn tiles_no_sense<'a, T: Tiles>(
    tiles: &'a T,
    palette: &'a Palette,
    tilesets: &'a Tilesets,
    textures: &'a TilesetTextures,
    selection: Option<&'a TileSelection>,
    overlay: Overlay,
) -> impl egui::Widget + 'a {
    tiles_with_sense(
        tiles,
        palette,
        tilesets,
        textures,
        selection,
        overlay,
        egui::Sense::empty(),
    )
}

/// Tile map, specifying the [`egui::Sense`] to use
pub fn tiles_with_sense<'a, T: Tiles>(
    tiles: &'a T,
    palette: &'a Palette,
    tilesets: &'a Tilesets,
    textures: &'a TilesetTextures,
    selection: Option<&'a TileSelection>,
    overlay: Overlay,
    sense: Sense,
) -> impl egui::Widget + 'a {
    move |ui: &mut egui::Ui| {
        tiles_ui(
            ui, tiles, palette, tilesets, textures, selection, overlay, sense,
        )
    }
}
