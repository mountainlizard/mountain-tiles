use egui::ahash::HashMap;
use egui::{
    load::{Bytes, BytesPoll, ImageLoadResult, ImageLoader, ImagePoll, LoadError, SizeHint},
    mutex::Mutex,
    ColorImage,
};
use image::ImageFormat;
use std::{mem::size_of, path::Path, sync::Arc, task::Poll};

#[cfg(not(target_arch = "wasm32"))]
use std::thread;

use crate::data::tilesets::TilesetMode;

type Entry = Poll<Result<Arc<ColorImage>, String>>;

#[derive(Default)]
pub struct TilesetImageLoader {
    cache: Arc<Mutex<HashMap<String, Entry>>>,
}

impl TilesetImageLoader {
    pub const ID: &'static str = egui::generate_loader_id!(TilesetImageLoader);
}

fn is_supported_uri(uri: &str) -> bool {
    let Some(ext) = Path::new(uri)
        .extension()
        .and_then(|ext| ext.to_str().map(|ext| ext.to_lowercase()))
    else {
        // `true` because if there's no extension, assume that we support it
        return true;
    };

    // Uses only the enabled image crate features
    ImageFormat::from_extension(ext).is_some_and(|format| format.reading_enabled())
}

fn is_supported_mime(mime: &str) -> bool {
    // some mime types e.g. reflect binary files or mark the content as a download, which
    // may be a valid image or not, in this case, defer the decision on the format guessing
    // or the image crate and return true here
    let mimes_to_defer = [
        "application/octet-stream",
        "application/x-msdownload",
        "application/force-download",
    ];
    for m in &mimes_to_defer {
        // use contains instead of direct equality, as e.g. encoding info might be appended
        if mime.contains(m) {
            return true;
        }
    }

    // Uses only the enabled image crate features
    ImageFormat::from_mime_type(mime).is_some_and(|format| format.reading_enabled())
}

/// Load a (non-svg) image.
///
/// You must also opt-in to the image formats you need with e.g.
/// `image = { version = "0.25", features = ["jpeg", "png", "gif", "webp"] }`.
///
/// # Errors
/// On invalid image or unsupported image format.
fn load_image_bytes(
    image_bytes: &[u8],
    mode: &TilesetMode,
) -> Result<egui::ColorImage, egui::load::LoadError> {
    let image = image::load_from_memory(image_bytes).map_err(|err| match err {
        image::ImageError::Unsupported(err) => match err.kind() {
            image::error::UnsupportedErrorKind::Format(format) => {
                egui::load::LoadError::FormatNotSupported {
                    detected_format: Some(format.to_string()),
                }
            }
            _ => egui::load::LoadError::Loading(err.to_string()),
        },
        err => egui::load::LoadError::Loading(err.to_string()),
    })?;

    let size = [image.width() as _, image.height() as _];
    let mut image_buffer = image.to_rgba8();

    for (_, _, pixel) in image_buffer.enumerate_pixels_mut() {
        mode.transform_color_slice(&mut pixel.0);
    }

    let pixels = image_buffer.as_flat_samples();

    // TODO(emilk): if this is a PNG, looks for DPI info to calculate the source size,
    // e.g. for screenshots taken on a high-DPI/retina display.

    Ok(egui::ColorImage::from_rgba_unmultiplied(
        size,
        pixels.as_slice(),
    ))
}

impl ImageLoader for TilesetImageLoader {
    fn id(&self) -> &str {
        Self::ID
    }

    fn load(&self, ctx: &egui::Context, uri: &str, _: SizeHint) -> ImageLoadResult {
        // Only loads uris starting with `tileset://`, for those we strip
        // the protocol and then load the resulting uri as an image.
        // There are three stages of guessing if we support loading the image:
        // 1. URI extension (only done for files)
        // 2. Mime from `BytesPoll::Ready`
        // 3. image::guess_format (used internally by image::load_from_memory)

        println!("TIL load uri {}", uri);

        // We will cache via the full uri, so retain it
        let full_uri = uri;

        // We only handle tileset protocol
        let uri = uri
            .strip_prefix("tileset://")
            .ok_or(LoadError::NotSupported)?;

        // Now split out into nested uri and encoded mode
        let (mode_json, uri) = uri.split_once("//").ok_or(LoadError::NotSupported)?;

        // Parse the mode
        let mode: TilesetMode =
            serde_json::from_str(mode_json).map_err(|_| LoadError::NotSupported)?;

        println!("TIL mode {:?}", mode);

        // (1)
        if uri.starts_with("file://") && !is_supported_uri(uri) {
            return Err(LoadError::NotSupported);
        }

        #[cfg(not(target_arch = "wasm32"))]
        #[expect(clippy::unnecessary_wraps, clippy::expect_used)] // needed here to match other return types
        fn load_image(
            ctx: &egui::Context,
            full_uri: &str,
            mode: TilesetMode,
            cache: &Arc<Mutex<HashMap<String, Entry>>>,
            bytes: &Bytes,
        ) -> ImageLoadResult {
            let full_uri = full_uri.to_owned();
            cache.lock().insert(full_uri.clone(), Poll::Pending);

            // Do the image parsing on a bg thread
            thread::Builder::new()
                .name(format!("egui_extras::ImageLoader::load({full_uri:?})"))
                .spawn({
                    let ctx = ctx.clone();
                    let cache = cache.clone();

                    let full_uri = full_uri.clone();
                    let bytes = bytes.clone();
                    move || {
                        log::trace!("ImageLoader - started loading {full_uri:?}");
                        let result = load_image_bytes(&bytes, &mode)
                            .map(Arc::new)
                            .map_err(|err| err.to_string());
                        log::trace!("ImageLoader - finished loading {full_uri:?}");
                        let prev = cache.lock().insert(full_uri, Poll::Ready(result));
                        debug_assert!(
                            matches!(prev, Some(Poll::Pending)),
                            "Expected previous state to be Pending"
                        );

                        ctx.request_repaint();
                    }
                })
                .expect("failed to spawn thread");

            Ok(ImagePoll::Pending { size: None })
        }

        #[cfg(target_arch = "wasm32")]
        fn load_image(
            _ctx: &egui::Context,
            full_uri: &str,
            mode: &TilesetMode,
            cache: &Arc<Mutex<HashMap<String, Entry>>>,
            bytes: &Bytes,
        ) -> ImageLoadResult {
            let mut cache_lock = cache.lock();
            log::trace!("started loading {full_uri:?}");
            let result = load_image_bytes(bytes, &mode)
                .map(Arc::new)
                .map_err(|err| err.to_string());
            log::trace!("finished loading {full_uri:?}");
            cache_lock.insert(full_uri.into(), std::task::Poll::Ready(result.clone()));
            match result {
                Ok(image) => Ok(ImagePoll::Ready { image }),
                Err(err) => Err(LoadError::Loading(err)),
            }
        }

        let entry = self.cache.lock().get(full_uri).cloned();
        if let Some(entry) = entry {
            match entry {
                Poll::Ready(Ok(image)) => Ok(ImagePoll::Ready { image }),
                Poll::Ready(Err(err)) => Err(LoadError::Loading(err)),
                Poll::Pending => Ok(ImagePoll::Pending { size: None }),
            }
        } else {
            match ctx.try_load_bytes(uri) {
                Ok(BytesPoll::Ready { bytes, mime, .. }) => {
                    // (2)
                    if let Some(mime) = mime {
                        if !is_supported_mime(&mime) {
                            return Err(LoadError::FormatNotSupported {
                                detected_format: Some(mime),
                            });
                        }
                    }
                    load_image(ctx, full_uri, mode, &self.cache, &bytes)
                }
                Ok(BytesPoll::Pending { size }) => Ok(ImagePoll::Pending { size }),
                Err(err) => Err(err),
            }
        }
    }

    fn forget(&self, uri: &str) {
        let _ = self.cache.lock().remove(uri);
    }

    fn forget_all(&self) {
        self.cache.lock().clear();
    }

    fn byte_size(&self) -> usize {
        self.cache
            .lock()
            .values()
            .map(|result| match result {
                Poll::Ready(Ok(image)) => image.pixels.len() * size_of::<egui::Color32>(),
                Poll::Ready(Err(err)) => err.len(),
                Poll::Pending => 0,
            })
            .sum()
    }

    fn has_pending(&self) -> bool {
        self.cache.lock().values().any(|result| result.is_pending())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_support() {
        assert!(is_supported_uri("https://test.png"));
        assert!(is_supported_uri("test.jpeg"));
        assert!(is_supported_uri("http://test.gif"));
        assert!(is_supported_uri("file://test"));
        assert!(!is_supported_uri("test.svg"));
    }
}
