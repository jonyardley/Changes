# Releasing to TestFlight

The CD lane: `.github/workflows/release-testflight.yml` builds a signed
Release .ipa (release-profile Rust core) and uploads it to TestFlight
internal testing. It runs on a `v*` tag or manual dispatch — never per-PR
(macOS runners are 10×-billed). Local equivalent: `just testflight`.

## One-time bootstrap (Jon)

Mirrors intrada's setup — the same App Store Connect API key and Apple
Distribution certificate can be reused; only the app id and profile are new.

1. **App Store Connect**: create the app record — bundle id
   `com.changes.app`, team `9S5FG4LQAF`.
2. **Certs repo**: create a PRIVATE repo (default the Matchfile expects:
   `jonyardley/changes-certificates`), or point `MATCH_GIT_URL` at an
   existing match repo — match stores per-app profiles side by side, and
   the Distribution cert is per-team, so reusing intrada's certs repo also
   works.
3. **Seed signing assets** (locally, once):

   ```bash
   rbenv shell 3.3.11   # system Ruby 2.6 is too old
   bundle install
   ASC_KEY_ID=… ASC_ISSUER_ID=… ASC_KEY_CONTENT_BASE64=… \
   MATCH_PASSWORD=… bundle exec fastlane match appstore
   ```

   CI only ever reads (`readonly: true`); this is the sole write.
4. **GitHub secrets** (repo → Settings → Secrets → Actions — same names as
   intrada, so copying values across is mechanical):

   | Secret | Contents |
   |---|---|
   | `ASC_KEY_ID` | App Store Connect API key id |
   | `ASC_ISSUER_ID` | ASC API issuer id |
   | `ASC_KEY_CONTENT_BASE64` | the .p8, base64-encoded |
   | `MATCH_GIT_URL` | certs repo HTTPS URL |
   | `MATCH_GIT_BASIC_AUTHORIZATION` | `base64("username:PAT")` with read access to the certs repo |
   | `MATCH_PASSWORD` | match encryption passphrase |

## Cutting a build

- Tag: `git tag v0.1.0 && git push origin v0.1.0`, or run the workflow
  manually from the Actions tab.
- Build number = the workflow run number (monotonic); marketing version =
  `MARKETING_VERSION` in `ios/project.yml` — bump it there when a release
  is more than a build.
- The .ipa is also attached to the run as an artifact for 14 days.

## Design decisions (inherited from intrada, kept deliberately)

- **match over Xcode automatic signing in CI**: deterministic, no Apple ID
  2FA on runners; the daily simulator loop keeps Automatic signing because
  the Fastfile flips the generated project to manual per-run.
- **Release Rust core**: debug cores are 10–100× slower in audio hot paths.
- **`CURRENT_PROJECT_VERSION` injected via xcargs**: CFBundleVersion is
  generated (`GENERATE_INFOPLIST_FILE=YES`); editing a partial plist with
  PlistBuddy doesn't work.
- **Exact-pinned fastlane + committed `Gemfile.lock`**: reproducible CI
  resolution (`bundler-cache: true` requires the lockfile).
