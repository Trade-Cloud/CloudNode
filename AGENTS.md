# AGENTS.md: guide for AI agents and contributors

CloudNode is a fast, open-source SSH client by TradeCloud for macOS, Linux and Windows. It is
written in **Rust only**: the desktop app is native [Slint](https://slint.dev), the console app is
[Ratatui](https://ratatui.rs) on Crossterm, and both share every crate below them. **No Tauri,
Electron or webview shells**, ever, and **no plugin system**: CloudNode is one compiled program per
interface, with no runtime layers to load.

**Local-first.** There is no CloudNode backend, no account, no cloud sync and no telemetry.
CloudNode opens no network connection except the SSH connections the user starts. Hosts, keys,
passwords and settings live only on the user's machine, in an encrypted vault.

The product name is always **CloudNode** in user-facing text (UI, window titles, About, installers,
release names, docs prose), and the maker is **TradeCloud**. Machine names stay lowercase: crates
(`cloudnode-*`), the binaries `cloudnode` (desktop) and `cloudnode-tui` (console), file names, and
the app id `com.tradecloud.cloudnode`. The repository is `TradeCloud/CloudNode`.

The project follows the conventions of ArtCraft's Craft apps (workspace layout, the never-crash
rules, attribution, the brand split), adapted to an SSH client.

## 0. Working with the maintainer

- **Agents do not run anything that executes or publishes CloudNode unless the maintainer asks
  for it in the current request:** no `cargo run`, no tests (`cargo test`, `cargo xtask ci`), no
  screenshots, no `git commit`, no `git push`. The maintainer runs and checks everything. Write
  the change, then list the exact commands to run (section 4).
- One request's permission covers that request only.

## 1. Workspace map

```text
crates/
  core      cloudnode-core      L0  pure data, no I/O: hosts, groups, key descriptions, colour tokens
  vault     cloudnode-vault     L1  the encrypted vault: master password, Argon2id, AEAD, atomic writes
  keys      cloudnode-keys      L1  SSH keys on this machine: reads ~/.ssh (public keys only) for Key Management
  ssh       cloudnode-ssh       L1  sessions on russh + Tokio: host keys, sign-in, PTY, resize
  term      cloudnode-term      L1  terminal emulation: escape sequences, cell grid, scrollback
  ui-slint  cloudnode-ui-slint  L2  the desktop interface: .slint layout (ui/), theme, terminal renderer
  ui-tui    cloudnode-ui-tui    L2  the console interface: Ratatui views
apps/
  cloudnode                         desktop shell: logging, start-up, exit
  cloudnode-tui                     console shell
xtask/                              cargo xtask ci | layers | version
assets/app-icon/                    the cheetah app icon renders (TradeCloud trademark), from packaging/icons.py
packaging/                          package.sh (release files per system), icons.py, macOS and Linux templates
.github/workflows/release.yml       every push to main: a draft release for macOS, Windows and Linux
docs/brand/                         TradeCloud marks and their license (not open source)
docs/releasing.md                   branches (develop, main) and cutting a release
```

**Layering is enforced** by `cargo xtask layers`: a crate may depend only on lower layers, `core`
depends on no workspace crate, and nothing below L2 uses Slint, Ratatui, Crossterm or winit. A new
crate under `crates/` must be registered in `xtask/src/layers.rs`. The apps stay thin: start-up,
logging and exit only; behaviour lives in the crates.

## 2. Golden rules

### Never crash (outranks feature work)

A dropped connection, a hostile server, a malformed key file, a corrupt vault, a wrong master
password or a full disk must produce an error the user can act on, never a panic. A crash can drop
every open session at once.

- **Non-test code never panics.** No `unwrap()`, `expect()`, `panic!`, `unreachable!`, `todo!` or
  `unimplemented!`. Return the crate's error type and propagate with `?`; use `ok_or(..)?`,
  `let … else`, `if let`, or `unwrap_or*` only where the fallback is truly correct.
- Every production crate root (`lib.rs`, each binary's `main.rs`, and `xtask`) carries
  `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]`.
  New crates start with it. Tests may unwrap (`clippy.toml`).
- **No `unsafe`.** The workspace sets `unsafe_code = "forbid"`. The one exception is
  `cloudnode-ui-slint`, which sets `deny` so that only its `mod generated` (Slint's generated
  code) can allow it; nothing written by hand there uses `unsafe`.
- **Input-derived numbers are hostile:** `get()` instead of `[i]`, slice strings only at char
  boundaries, checked or saturating arithmetic for sizes and offsets, cap allocations.
- **Don't cascade:** handle lock poisoning (`lock().unwrap_or_else(PoisonError::into_inner)`) and
  treat thread and task joins as `Result`s.
- **Prove it:** every crash fix lands with a regression test that panicked before the fix.

### The server is hostile

Everything an SSH server sends is untrusted: banners, key exchange data, channel data, window
sizes, escape sequences, file names. Bound every length before allocating, never let it panic the
client, and never let terminal output reach the user's own machine (no clipboard writes, file
writes or URL opens triggered by escape sequences without the user's action).

- Host keys are checked against CloudNode's `known_hosts` in the vault. An unknown key asks the
  user with its SHA-256 fingerprint; a changed key refuses to connect until the user decides.
  Nothing is accepted silently.

### Secrets

- Private keys, passwords and the master password never appear in logs, `Debug` output, error
  messages, crash reports or files outside the vault. Types that hold secrets implement a
  redacting `Debug` and zeroize on drop.
- Files that hold secrets are written through `cloudnode_vault::atomic::write` (owner-only on Unix).
- Reading `~/.ssh` (`cloudnode-keys`) touches public key files only. Of any other file it reads
  the first few bytes, to tell a private key by its header; private key contents are read only
  when a key is actually used, and never kept beyond that.
- No hand-rolled cryptography. Use the RustCrypto crates named in section 3, with parameters kept
  in one place in `cloudnode-vault`.

### Look and feel

- Apple-like and calm: a restrained dark theme, careful type, generous spacing, rounded corners
  (6, 8 and 12 px), and 1 px hairlines instead of shadows.
- TradeCloud's palette, in layers (the base constants in `crates/core/src/theme.rs`):

  | Colour | Constant | Role |
  |---|---|---|
  | `#171e28` | `BACKGROUND` | The back: the left panel and the middle of the window |
  | `#232b36` | `SURFACE` | The main colour in front: the session panel (and the terminal), menus, popovers |
  | `#2a3845` | `ELEMENT` | Elements in the foreground: fields, buttons, the selected row, hairlines |
  | `#3bbae3` | `ACCENT` | Selected icons (the open panel), focus, the active session, primary buttons |
  | `#818898` | `TEXT` | Icons and text |

  The hover, strong-text and strong-line tints are derived from these in `Tokens::DARK`. The
  accent is the only bright colour: never use it for decoration.
- Colours live only in `crates/core/src/theme.rs` (`Tokens`, shared by the window, the TUI and
  the terminal renderer); sizes, radii and type live only in `crates/ui-slint/ui/theme.slint`
  (`Metrics`). **Never write a colour literal anywhere else.**
- The title bar holds the native macOS window buttons, the session tabs and a minimal set of
  actions. No global search bar, user profiles or debug buttons. The sidebar holds a compact host
  and key search, the host lists (Favorites, Connected, groups) and Key Management at the bottom.
  The rest of the window belongs to the terminal.

### Languages

Every user-facing string in `.slint` files goes through `@tr("…")`; the English text is the key.
Product names (CloudNode, TradeCloud, Slint) are never translated. The TUI gets the same catalogs
when localisation lands; until then its strings stay English and short.

### Dependencies and clean room

- **Pure Rust by default.** A dependency that builds C or C++ code needs the maintainer's approval,
  with the reason in a comment beside it in `Cargo.toml`.
- Dependencies must be permissively licensed (MIT, Apache-2.0, BSD, ISC, Zlib, MPL-2.0). Slint is
  the one exception, used under its royalty-free license (see `NOTICE`): every release needs the
  Slint attribution in the About window or on its download page.
- Never copy code from GPL, LGPL or AGPL projects (for example libssh, GNOME's VTE, Konsole).
  Implement from the RFCs (4251–4254, 4256, 8709, 8332, 8308),
  the OpenSSH protocol notes and xterm's control sequence documentation.

### Assets and brand

1. **Every asset gets a row in `ATTRIBUTION.md`** (path, title, author, source, license) in the
   same change, with its license file beside it when the license requires one.
2. The TradeCloud name, logo and the CloudNode cheetah mark (`docs/brand/`, and the app icon made
   from it in `assets/app-icon/`) are trademarks, **not MIT/Apache**
   (`docs/brand/LICENSE-brand.txt`). Use them exactly as supplied; never redraw, recolour or
   derive from them.
3. The app icon is a running cheetah, hand-drawn in ink with TradeCloud orange, on a light
   rounded tile. Every render in `assets/app-icon/` comes from TradeCloud's master drawing through
   `packaging/icons.py`; never edit them by hand, run the script again.
4. When fetching anything, use a generic `CloudNode/<version>` User-Agent and never put a
   person's name, email or other personal details in requests.

## 3. Stack decisions

| Area | Choice | Why |
|---|---|---|
| Desktop UI | Slint 1.18 | Native, declarative, light; royalty-free license for desktop apps (attribution required) |
| Console UI | Ratatui 0.30 + Crossterm | The standard Rust TUI stack, pure Rust, works in every terminal |
| SSH | `russh` ≥ 0.62.5 on Tokio | Pure Rust and async. `ssh2`/libssh2 pulls in C and OpenSSL, which complicates three-platform builds |
| Keys | RustCrypto `ssh-key` 0.6 (parsing only, `std` + `ecdsa`); `russh::keys` once sessions land | Reads OpenSSH public keys and SHA-256 fingerprints for every key type, pure Rust; avoids using the `rsa` crate directly (open timing advisory, Marvin) |
| Vault crypto | Argon2id + XChaCha20-Poly1305 + `zeroize` (RustCrypto) | A KDF for the master password, a misuse-resistant AEAD with random nonces, pure Rust (unlike `ring`'s C and assembly) |
| Storage | One encrypted vault file, atomic writes | Hundreds of hosts fit easily; no C. `sled` is unmaintained; `rusqlite` (bundled SQLite) can come later if queries or history need it |
| Terminal | `alacritty_terminal` behind `cloudnode-term` | Mature VT parser and grid, Apache-2.0; the GUI draws the grid into a pixel buffer |
| Packaging | Our own scripts in `packaging/` | Like the Craft apps: `.app` + DMG, MSI + portable zip, AppImage, `.deb`, `.rpm`. `cargo-bundle` covers only part of that |

## 4. Before you finish a task

The gate (the maintainer runs it; agents list it, see section 0):

```sh
cargo fmt --all
cargo xtask ci              # fmt --check, clippy -D warnings, tests, layers
```

Or piecemeal: `cargo test -p <crate>`, `cargo clippy -p <crate> --all-targets -- -D warnings`,
`cargo xtask layers`. On Linux the desktop crates need Slint's system libraries (see README).

## 5. Branches and releasing

- **`develop`** is where work happens. It has **no automatic builds**: never add a workflow that
  runs on it.
- **`main`** is for releases. Every push to `main` runs `.github/workflows/release.yml`, which
  builds macOS, Windows and Linux with `packaging/package.sh` and uploads the files and
  `SHA256SUMS.txt` to a **draft** GitHub Release `CloudNode v<version>`. A maintainer publishes it
  by hand; nothing is ever published automatically.
- The version lives only in `[workspace.package] version` in the root `Cargo.toml`
  (`cargo xtask version`, `cargo xtask version set X.Y.Z[-pre]`). Bump it on `develop` before
  merging to `main`: a version that is already published stops the release run.
- Release builds use `--locked`, so `Cargo.lock` is committed and kept current.
- Keep the release file names exact (`docs/releasing.md`); the README links to them.

Details: [`docs/releasing.md`](docs/releasing.md). Signing, notarization and installers are
milestone M7 (`ROADMAP.md`).

## 6. Parallel agents

- Use your own target dir (`CARGO_TARGET_DIR=target/agent-<name>`) and edit only the files you own.
- Keep every `Cargo.toml` valid at all times: the `crates/*` and `apps/*` globs mean one broken
  manifest breaks everyone's build. Write manifests to a temporary file and `mv` them into place.
- If someone else's in-progress edit breaks the build, wait and retry; don't fix their files.
