extern crate embed_resource;
fn main() {
    // On Windows, embed the `icon.ico` linked from `icon.rc` (both in project root), into our .exe
    // For rc contents, see https://learn.microsoft.com/en-us/windows/win32/menurc/icon-resource
    // For embed_resource docs, see https://crates.io/crates/embed-resource - we used the manifest example, but using
    // different rc content and resource file.
    // This page has a nice overview https://gamesbymason.com/2021/01/05/setting-a-rust-windows-exe-icon/
    // Note that the icon resource name in `icon.rc` seems to be arbitrary, apparently if there are multiple
    // icon resources, the one whose name comes first alphabetically is used
    println!("cargo:rerun-if-changed=icon.rc");
    println!("cargo:rerun-if-changed=icon.ico");
    embed_resource::compile("icon.rc", embed_resource::NONE)
        .manifest_optional()
        .unwrap();
}
