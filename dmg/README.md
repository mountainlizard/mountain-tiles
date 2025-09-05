# README

The background for the dmg window is a little fiddly - it needs to be the same width as the window (see `Cargo.toml`, `window-size` setting), however it should be a bit less than the window height, since the window height includes the title bar, but the background only covers the actual active area of the window. The safest approach is probably to use a transparent background, and make the height of the image only cover what you need (e.g. to put a "drag icon here" arraw in the background).

Once you've picked a size, replace the two png's with images at that size (for @1x version) and twice that size (for the @2x version), then run `./create-tiff.sh`.
