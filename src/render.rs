use crate::{
    data::palette::Palette,
    data::png::PngExportSettings,
    data::tiles::Tiles,
    data::tilesets::{TilesetId, Tilesets},
    ui::tileset_textures::TilesetTextures,
};
use egui::ahash::{HashMap, HashMapExt};
use eyre::{bail, eyre, Result};
use image::Pixel;
use image::{imageops::resize, GenericImage, GenericImageView, ImageReader, RgbaImage};

type TilesetImages = HashMap<TilesetId, RgbaImage>;

fn cache_tileset_images<T: Tiles>(
    tiles: &T,
    tilesets: &Tilesets,
    textures: &TilesetTextures,
) -> Result<TilesetImages> {
    // Cache tileset images, and check they all have tile_size matching the map
    let mut tileset_images = HashMap::new();
    for tileset in tilesets.iter() {
        if tileset.tile_size != tiles.tile_size() {
            bail!(
                "Cannot export as an image unless all tilesets have the same tile size as the map."
            )
        }

        let path = textures
            .path_for_tileset(tileset)
            .ok_or(eyre!("No path for tileset"))?;
        let dynamic_image = ImageReader::open(path)?.decode()?;
        let image = dynamic_image.into_rgba8();
        tileset_images.insert(tileset.id(), image);
    }
    Ok(tileset_images)
}

pub fn render_tiles<T: Tiles>(
    tiles: &T,
    palette: &Palette,
    tilesets: &Tilesets,
    textures: &TilesetTextures,
    settings: &PngExportSettings,
) -> Result<RgbaImage> {
    let size = tiles.pixel_size_unscaled();
    let scale = settings.scale;
    let transparent = settings.transparent;

    // Cache tileset images, and check they all have tile_size matching the map
    let tileset_images = cache_tileset_images(tiles, tilesets, textures)?;

    let mut image = RgbaImage::new(size.w, size.h);

    if !transparent {
        let bg = tiles.background().into();
        for pixel in image.enumerate_pixels_mut() {
            *pixel.2 = bg;
        }
    }

    for layer_index in (0..tiles.layer_count()).rev() {
        let opacity = tiles.layer_opacity(layer_index);
        for tileset in tilesets.iter() {
            let tile_set_size = tileset.size_in_tiles;
            let tile_size = tileset.tile_size;

            for grid_pos in tiles.map_positions() {
                if let Some(ref tile) = tiles.tile(layer_index, grid_pos) {
                    if tile.source.tileset_id == tileset.id() {
                        let tileset_image = tileset_images
                            .get(&tileset.id())
                            .ok_or(eyre!("Cannot export, map uses an invalid tileset"))?;
                        let color = tile
                            .color
                            .as_user_color(palette)
                            .with_optional_opacity(opacity)
                            .as_slice();
                        let tile_pos =
                            tile_set_size.pos_from_linear_index(tile.source.tile_index.index());
                        let tile_pixel_pos = tile_pos * tile_size;

                        let tile_dest_pixel_pos = grid_pos * tile_size;
                        let tile_source_image = tileset_image.view(
                            tile_pixel_pos.x,
                            tile_pixel_pos.y,
                            tile_size.w,
                            tile_size.h,
                        );
                        let mut tile_dest_image = image.sub_image(
                            tile_dest_pixel_pos.x,
                            tile_dest_pixel_pos.y,
                            tile_size.w,
                            tile_size.h,
                        );

                        // Copy the relevant area of the tileset image to the final image, applying color
                        for y in 0..tile_size.h {
                            for x in 0..tile_size.w {
                                let mut sx = x;
                                let mut sy = y;
                                if tile.transform.mirror_x() {
                                    sx = tile_size.w - 1 - sx;
                                }
                                if tile.transform.mirror_y() {
                                    sy = tile_size.h - 1 - sy;
                                }
                                if tile.transform.swap_xy() {
                                    (sx, sy) = (sy, sx);
                                }

                                let mut src = tile_source_image.get_pixel(sx, sy);

                                for (s, c) in src.0.iter_mut().zip(color.iter()) {
                                    let mut v: u32 = *s as u32;
                                    v = (v * *c as u32) / 255;
                                    *s = v as u8;
                                }

                                let mut dst = tile_dest_image.get_pixel(x, y);
                                dst.blend(&src);
                                tile_dest_image.put_pixel(x, y, dst);
                            }
                        }
                    }
                }
            }
        }
    }

    let image = resize(
        &image,
        size.w * scale,
        size.h * scale,
        image::imageops::FilterType::Nearest,
    );

    Ok(image)
}
