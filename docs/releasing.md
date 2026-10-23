# Branches and releases

## Branches

| Branch | What it is for | Automatic builds |
|---|---|---|
| `develop` | Everyday work. Every change lands here first. | **None** |
| `main` | Releases only. A push here means "build this as the next release". | **Draft release** for macOS, Windows and Linux |

Work on `develop` (or a branch off it), and merge `develop` into `main` when a version is ready.

## Cutting a release

1. On `develop`, set the new version. It lives only in `[workspace.package] version` in the root
   `Cargo.toml`:

   ```sh
   cargo xtask version              # prints the current version
   cargo xtask version set 0.2.0    # or 0.2.0-rc.1 for a pre-release
   ```

   Build once so `Cargo.lock` picks up the new version, and commit both files. The release build
   uses `--locked`, so `Cargo.lock` must be committed and up to date.

2. Merge `develop` into `main` and push `main`.

3. [`.github/workflows/release.yml`](../.github/workflows/release.yml) then:
   - reads the version and creates the **draft** release `CloudNode v<version>` (or reuses the
     draft if one for that version exists, replacing its files);
   - builds on macOS, Windows and Linux with [`packaging/package.sh`](../packaging/package.sh)
     and uploads the files to the draft;
   - writes `SHA256SUMS.txt` for every file and uploads it too.

   If `v<version>` is already published, the run stops at the first step: bump the version first.

4. On GitHub, open **Releases**, check the draft's files, write the notes and **Publish**.
   Publishing creates the `v<version>` tag on the commit the draft was built from. Until then
   nothing is public.

## Release files

| Platform | File | Contents |
|---|---|---|
| macOS 11+ | `cloudnode-<ver>-macos-universal.dmg` | `CloudNode.app`, Apple silicon and Intel |
| macOS 11+ | `cloudnode-tui-<ver>-macos-universal.tar.gz` | The console app |
| Windows 10/11 | `cloudnode-<ver>-windows-x64-portable.zip` | `cloudnode.exe` and `cloudnode-tui.exe` |
| Linux (glibc 2.35+) | `cloudnode-<ver>-linux-x86_64.tar.gz` | `bin/`, `share/applications`, `share/icons` |
| All | `SHA256SUMS.txt` | SHA-256 of every file above |

Keep these names exact: the README's download table and any future updater rely on them.

## Not yet

- Signing with a developer certificate and Apple notarization (macOS builds are ad-hoc signed,
  so Gatekeeper asks once; Windows SmartScreen may warn).
- MSI, AppImage, `.deb` and `.rpm` packages, Linux on ARM, Windows on ARM.

Both are milestone M7 in [`ROADMAP.md`](../ROADMAP.md).
