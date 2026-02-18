# Continuous integration and packaging

For the most part, the CI set up in github actions should just run, however it does expect to have various secrets set up for Apple signing/notarization. See the [github actions howto](https://docs.github.com/en/actions/how-tos/deploy/deploy-to-third-party-platforms/sign-xcode-applications) for the required secrets - see `rust.yml` in the github workflows for details.

- `BUILD_CERTIFICATE_BASE64` contains base64-encoded .p12 for an Apple Signing Certificate.
- `P12_PASSWORD` contain the password for the .p12 file.
- `KEYCHAIN_PASSWORD` a random string used as the keychain password for the keychain created on the agent as part of build.

We've omitted the following secrets from the how-to, since we don't build mobile versions:

- `BUILD_PROVISION_PROFILE_BASE64` contains base64-encoded provisioning profile.

Note that if the name of the developer changes, the `signing-identity` in `Cargo.toml` will need to be updated - this should be the name of the certificate in the .p12 file stored as a secret.

In order to perform a release, you also need:

- `RELEASE_TOKEN` contains a github personal access token with permissions to read and write contents, this is used to create a release and attach assets.

To trigger a release, make sure you are on main branch and on the correct commit (most likely the head), and that the `Cargo.toml` package version in that commit matches the version number of the release, then add a tag to the repo starting with `v`, then the version number, e.g.:

```bash
git tag -a "v0.1.6" -m "Release v0.1.6"
git push --tags "origin"
```

This will trigger the `rust.yml` workflow, and since the event is a `push` and the ref starts with `refs/tags/v` this will also run the `release` job, which creates a release named after the tag, and uploads all artifacts to the release.

## Debian (.deb) packaging

See `Cargo.toml` file, in the `[package.metadata.packager.deb]` section.
This includes additional files for the .deb package - the `files` field is a map, where each key is the file path relative to the root of the project, and each value is the location in the resulting .deb package. All source files are in the `deb` directory of the project.

In summary, there is:

1. `x-mountain-tiles.xml`, which is installed to the `usr/share/mime/packages` directory on the user's system. This should be detected by the package manager to update the mime database. This allows us to associate a mime type with our application, linking it to file extension(s).

2. A set of application icons installed to `/usr/share/icons/hicolor`, which should in theory be used for the application, although this doesn't always seem to work. The package manager should detect these and update the icon database.

Finally, the package manager should detect the `.desktop` file, as created by `cargo-packager`, and use it to update the desktop database.

For more details, see [freedesktop docs](https://specifications.freedesktop.org/shared-mime-info-spec/latest/ar01s02.html), we also used existing deb files as a reference.

## References

- [Helpful guide on signing and notarizing on macOS](https://scriptingosx.com/2021/07/notarize-a-command-line-tool-with-notarytool/)
