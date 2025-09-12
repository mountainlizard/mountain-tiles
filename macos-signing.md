# Signing on macOS

To sign on macOS (for distribution of a `.dmg` file outside the App Store), you will need to:

1. Make sure you have a valid, paid membership of the [Apple Developer Program](https://developer.apple.com/programs/).
2. Install and run XCode on macOS.
3. Go to the main "Xcode" menu, and select "Settings".
4. Go to the "Accounts" tab, make sure you are signed into your Apple Account (add it and/or sign in)
5. Click "Manage Certificates".
6. If you don't already have one, create a "Developer ID Application" certificate (using "+" dropdown in bottom left).
7. You should now see a "Developer ID Application" certificate in the list, note the "Name" and "Creator" for this. You may also be able to get these from `security find-identity -p basic -v` and copying the part within double quotes (this will also include the "Developer Team ID" in brackets).
8. You can now set the `signing_identity` in `Cargo.toml` to `Name: Creator`, using the name and creator strings from the certificate you generated, e.g. something like `"Developer ID Application: DEVELOPER_NAME"`. You can also use the version with the Team ID in brackets, e.g. `"Developer ID Application: DEVELOPER_NAME (ABCD123456)"`.

Now when you package with `cargo packager --release`, you should get a signed `.app` and `.dmg`. The first time you build you will be prompted for access to the developer certificate - enter your password, and either "Allow" to allow once, or "Always Allow" to not be prompted multiple times per build. You should also see a `WARN Skipping app notarization` message in the terminal since we've not set up notarization yet.

To notarize on macOS (to remove/reduce warnings when users use the packaged app):

1. First we need to create an app-specific password for the `notarytool` util we will use to notarise. This is a reasonable way to run locally, but note that there is a better approach using an API Key, see [Signing and Notarizing on CI](#signing-and-notarizing-on-ci). The following steps are based on [these apple docs](https://support.apple.com/en-us/102654):
2. Sign in to the [Apple Account portal](https://account.apple.com) using the Apple ID for your developer account.
3. Go to the "Sign-in and Security" section, and select "App-Specific Passwords"
4. Click "Generate an app-specific password" (ignore that the dialog talks about apps not provided by Apple, this is a special case).
5. Provide a name, e.g. "notarytool"
6. Follow the instructions - for me this involved being shown a dialog saying you need to sign in again with password and 2FA, I clicked the button for this, and then signed in specifically NOT using a passkey, but providing my password and code, then I went back to "App-Specific Passwords" and tried again - this time it worked.
7. Make sure to leave the password screen open (or copy the password) long enough to complete the remaining steps, specifically both steps 9 and 14.
8. We'll now run a command to get `notarytool` to store the password for its own use - make sure to replace the Apple ID and Team ID with your own values below. The `--apple-id` parameter is your Apple Account used for your developer program membership (i.e. an email), and the `--team-id` is the code displayed in brackets by `security find-identity -p basic -v`, also found under "Membership details"->"Team ID" when signed in to your [Apple Developer Account](https://developer.apple.com/account):

   ```bash
   xcrun notarytool store-credentials --apple-id "name@example.com" --team-id "ABCD123456"
   ```

9. This will prompt for some details, for the keychain profile name just use something you will remember, e.g. "notarytool-APPLEID" using your Apple ID email, it will also request the password from step 7.
10. At this point, you should be able to try the notarization process from the command line, first make sure you've built a `.dmg` using the `cargo packager --release`, this should build and sign the `.dmg` as long as you've set up signing above. Then cd to the `target/release` dir, and submit the dmg for notarisation (replace the `.dmg` filename as appropriate):

    ```bash
    xcrun notarytool submit MountainTiles_0.1.0_aarch64.dmg --keychain-profile "notarytool-APPLEID" --wait
    ```

11. This may take a while, and should show a submission ID, then upload progress, then waiting for processing, and finally if all is well "Processing complete" with "status: Accepted".
12. At this point, there should be a record of your `.dmg` being ok to install, on Apple's servers - we can attach this to the `.dmg` itself with `xcrun stapler staple MountainTiles_0.1.0_aarch64.dmg` - this should display a message that the action worked. Note we're just notarising the dmg to test that our credentials are valid. If we wanted to do the signing/notarization without `cargo-packager`, we would instead need to sign, notarize and staple the app bundle, then produce a dmg, luckily `cargo-packager` will do this for us.
13. Finally, we can check the `.dmg` file with `spctl --assess -vv --type install MountainTiles_0.1.0_aarch64.dmg`, which should show "accepted", with "source" referring to a "Notarized Developer ID", and "origin" showing the "Developer ID Application" you used.
14. We can now try building and notarising using `cargo-packager` by passing env variables to it - make sure to replace the Apple ID and Apple Team ID with yours, and the password with the app-specific password from step 7:

    ```bash
    APPLE_ID='name@example.com' APPLE_TEAM_ID='ABCD123456' APPLE_PASSWORD='APP-SPECIFIC-PASSWORD' cargo packager --release
    ```

    On more recent versions of cargo-packager you can just run:

    ```bash
    APPLE_KEYCHAIN_PROFILE='notarytool-APPLEID' cargo packager --release
    ```

    This will use the keychain profile you set up in step 8, to avoid the need to specify ids and password in env vars.

15. Note that this should notarise the `.app`, but the `.dmg` will only be signed, not notarized (although the app inside it will be notarized). According to an Apple engineer in [this discussion on the Apple developer forums](https://developer.apple.com/forums/thread/650288) this is valid, only the app needs to be notarised, the dmg doesn't.
16. [Test the app will install](https://developer.apple.com/forums/thread/130560) - note this needs to be on a different copy of macOS, the linked post recommends using a vm that is reset after every test. For a quick check, try `syspolicy_check distribution MountainTiles.app` in the release dir, although as noted in that post this is not as reliable.

## Signing and Notarizing on CI

See main `README.md` in this project for the CI setup - for signing we essentially we just have to export the Developer Certificate as a .p12, then encode as base64 (`base64 -i BUILD_CERTIFICATE.p12 | pbcopy`), and provide that base64 encoding plus the .p12 file's password as secrets.

For notarisation, we want to use an API key, specifically an "App Store Connect API key", to avoid having to put an account password in github, even as a secret, since the password can be used for more than notarising applications.

The [docs page for creating api keys](https://developer.apple.com/documentation/appstoreconnectapi/creating-api-keys-for-app-store-connect-api) gives a reasonable summary and confirms that "App Store Connect API keys are unique to the App Store Connect API and you can’t use them for other Apple services.". However the page doesn't fill in all the details - the process we followed was:

1. Log in to [App Store Connect](https://appstoreconnect.apple.com/login)
2. Follow the instructions to select "Users and Access", and then select the "Integrations" tab, then select "App Store Connect API" in the left column.
3. At this point the docs are lacking - you may not see a "Team Keys" tab if you instead get the message "Permission is required to access the App Store Connect API. You can request access on behalf of your organization." with a "Request Access" button. If that's the case, click "Request Access" button and read/agree to text.
4. Select the "Team Keys" tab (note that we need a Team key because individual keys cannot be used with `notarytool`)
5. Click "Generate API Key" or the "Add (+)" button.
6. We name the key "notarisation (developer)" since that's all we plan to do with it and what role it has (see next step for what the role means).
7. We now need to choose a role - the [the permissions for each role](https://developer.apple.com/help/account/access/roles/) show us that the role with the least permissions that still has full access to "Notarize software" is "Developer", so we choose that role.
8. Click "Generate", and we can now see the key in the table, including the KEY ID, which looks like the example id in the docs, so we'll store that somewhere secret.
9. Also notice that the Issuer ID is shown above the table, so grab that and keep it secret and safe too.
10. Now we get ONE CHANCE to download the private half of the key, clicking the "Download" link. Thankfully this gives us a .p8 file as referenced in the notarytool docs, so that's good! Make sure to store this somewhere safe and secret (NOT IN A REPO!)

We can now see from [this page](https://developer.apple.com/documentation/technotes/tn3147-migrating-to-the-latest-notarization-tool) that we need the following parameters for `notarytool`, which is called by `cargo-packager`:

- `ISSUER_UUID` is your App Store Connect API key issuer, this is a long hexadecimal UUID string with dashes, something like `1234abcd-1234-abcd-1234-1234567890ab`
- `API_KEY` is your App Store Connect API key ID, this is a shorter alphanumeric string, something like `ABCD123456`
- `PATH_TO_KEY` is the path to the key’s .p8 file.

These are then called with `xcrun notarytool … --issuer ISSUER_UUID --key-id API_KEY --key PATH_TO_KEY`

We can test using `xcrun notarytool history …` with the parameters above to check they are accepted.

We can also try out these values with cargo-packager locally, something like the following (assuming the p8 file is still in your downloads) - use your own key id and issuer id, the ones below are just from the Apple docs example:

```bash
APPLE_API_KEY=ABCD123456 APPLE_API_ISSUER=1234abcd-1234-abcd-1234-1234567890ab APPLE_API_KEY_PATH=~/Downloads/AuthKey_ABCD123456.p8 cargo packager --release
```

This should give `INFO Notarizing Finished with status Accepted` somewhere in the output (it may take a while).

Now we need to get this to run on CI - we'll follow the same exact approach used for the developer certificate used for signing based on the [github actions howto](https://docs.github.com/en/actions/how-tos/deploy/deploy-to-third-party-platforms/sign-xcode-applications). So we encode the .p8 file as base64, and provide the key id and issuer id as secrets (not sure they actually need to be secret but just to be safe), and then decode the .p8 file to a temporary file on the agent and point to it with the key path.

Encode the .p8 file:

```bash
base64 -i AuthKey_ABCD123456.p8 | pbcopy
```

Then paste this into a secret `API_KEY_P8_BASE64`.

We named the other secrets after the required cargo-packager env vars, `APPLE_API_KEY` and `APPLE_API_ISSUER`, so we just need to pull them from secrets into env vars before running `cargo-packager` - see the github workflow `./github/workflows/rust.yml` for more details.
