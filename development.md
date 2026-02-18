# Development

This project was created using the [eframe template](https://github.com/emilk/eframe_template/)

## Running locally

Make sure you are using the latest version of stable rust by running `rustup update`.

On Linux you need to first run:

`sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev`

On Fedora Rawhide you need to run:

`dnf install clang clang-devel clang-tools-extra libxkbcommon-devel pkg-config openssl-devel libxcb-devel gtk3-devel atk fontconfig-devel`

To run directly:

```bash
cargo run --release
```

To watch for changes and kill then restart application, install [bacon](https://dystroy.org/bacon/) and then run:

```bash
bacon run
```

## Packaging

Packages are built using [cargo-packager](https://github.com/crabnebula-dev/cargo-packager).

Install this using:

```bash
cargo install cargo-packager --locked
```

You can then package for your current platform using:

```bash
cargo packager --release
```

If packaging on macOS, this may fail if you have the `.dmg` file created by a previous build still mounted - unmount it and run again.

To sign and notarise the application bundle on macOS, see [Signing on macOS](macos-signing.md).

Packaging has been tested as:

1. `.app` and `.dmg` on macOS (Arm64 on M1, x64 tested only via Rosetta on M1)
2. `.deb`, `.tar.gz` and `.AppImage` on Linux (pop_os on Intel, Raspberry Pi OS (KDE) on arm64)
3. `.exe` installer on Windows (Intel)
