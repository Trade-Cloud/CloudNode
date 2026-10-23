<p align="center">
  <img alt="CloudNode: a running cheetah" src="assets/app-icon/hicolor/256x256/apps/com.tradecloud.cloudnode.png" width="160">
</p>

<h1 align="center">CloudNode</h1>

<p align="center">
  <b>A fast, open-source SSH client by TradeCloud, built from scratch in pure Rust.</b><br>
  Instant startup, secure key management, and a unified TUI and GUI:<br>
  your servers in a calm native window, or right in your terminal.
</p>

<p align="center">
  <img alt="100% Rust" src="https://img.shields.io/badge/100%25-Rust-b7410e?style=flat-square&logo=rust">
  <img alt="macOS · Windows · Linux" src="https://img.shields.io/badge/macOS%20%C2%B7%20Windows%20%C2%B7%20Linux-native-26282e?style=flat-square">
  <img alt="License: MIT OR Apache-2.0" src="https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-3a3a3a?style=flat-square">
  <img alt="Local-first, no telemetry" src="https://img.shields.io/badge/local--first-no%20telemetry-3a3a3a?style=flat-square">
  <img alt="Status: pre-alpha" src="https://img.shields.io/badge/status-pre--alpha-3bbae3?style=flat-square">
</p>

<p align="center">
  <a href="https://github.com/TradeCloud/CloudNode/releases/latest"><b>Download CloudNode</b></a> ·
  <a href="ROADMAP.md">Roadmap</a> ·
  <a href="SECURITY.md">Security</a>
</p>

> [!NOTE]
> **CloudNode is local-first.** No backend, no account, no cloud sync and no telemetry. Your
> hosts, keys and passwords never leave your machine, and CloudNode connects only to the servers
> you open.

<p align="center">
  <a href="#what-it-does">What it does</a> ·
  <a href="#features">Features</a> ·
  <a href="#two-apps-one-cloudnode">Two apps</a> ·
  <a href="#downloads">Downloads</a> ·
  <a href="#build-from-source">Build from source</a> ·
  <a href="#license-and-credits">License</a>
</p>

## What it does

CloudNode is an SSH client for developers and system administrators who want it to open
instantly, keep their keys safe on their own machine, and stay out of the way. The window is
mostly terminal: session tabs in the title bar, a slim sidebar with your hosts and a compact
search, and Key Management for your Ed25519 and RSA keys at the bottom of the sidebar. No
oversized panels, no global search bar, no profile buttons.

## Features

<table>
  <tr>
    <td width="33%" valign="top">
      <h3>⚡ Instant start</h3>
      One native Rust program per interface. No Electron, no webview, no plugin runtime to load: CloudNode is up before you reach for the keyboard.
    </td>
    <td width="33%" valign="top">
      <h3>🔐 Your keys stay yours</h3>
      Hosts, keys and passwords live in a local vault encrypted with your master password (Argon2id and XChaCha20-Poly1305). Nothing is synced anywhere.
    </td>
    <td width="33%" valign="top">
      <h3>🖥️ Window or terminal</h3>
      The desktop app and <code>cloudnode-tui</code> share the same hosts, keys and look, so you can work from a laptop or over SSH on a server.
    </td>
  </tr>
  <tr>
    <td width="33%" valign="top">
      <h3>🗂️ Hosts at hand</h3>
      Favorites, Connected and your own groups in the sidebar, with one search over hosts and keys.
    </td>
    <td width="33%" valign="top">
      <h3>🎨 Calm by design</h3>
      A restrained dark blue-grey theme in the spirit of macOS, careful type and spacing, and one sky-blue accent only where it matters.
    </td>
    <td width="33%" valign="top">
      <h3>🛡️ Built not to crash</h3>
      A hostile server, a damaged key file or a full disk gives an error you can act on, never a lost session. Host keys are checked, never accepted silently.
    </td>
  </tr>
</table>

## Two apps, one CloudNode

| | App | For |
|:-:|---|---|
| 🪟 | **`cloudnode`** | The desktop app: a native [Slint](https://slint.dev) window with tabs, sidebar and terminal |
| ⌨️ | **`cloudnode-tui`** | CloudNode in the console ([Ratatui](https://ratatui.rs)): the same layout and colours, for servers and the keyboard-only |

Both are built from the same crates, so a host or key added in one is there in the other.

## Downloads

Download the build for your computer from the
[latest release](https://github.com/TradeCloud/CloudNode/releases/latest), or browse
[all releases](https://github.com/TradeCloud/CloudNode/releases). Every release has a
`SHA256SUMS.txt` to check the files against. `<ver>` is the version number, for example `0.1.0`.
The first release is on its way; until it is out, [build from source](#build-from-source).

**macOS** (11 Big Sur or later)

| File | What it is |
|---|---|
| `cloudnode-<ver>-macos-universal.dmg` | The app for Apple silicon and Intel Macs. Drag CloudNode to Applications |
| `cloudnode-tui-<ver>-macos-universal.tar.gz` | The console app: unpack it and put `cloudnode-tui` on your `PATH` |

The builds are not notarized yet: the first time, macOS asks before opening CloudNode. Choose
**System Settings → Privacy & Security → Open Anyway**.

**Windows** (10 and 11, 64-bit)

| File | What it is |
|---|---|
| `cloudnode-<ver>-windows-x64-portable.zip` | No install: unzip and run `cloudnode.exe`, or `cloudnode-tui.exe` in a terminal |

The builds are not signed yet, so SmartScreen may warn: choose **More info → Run anyway**.

**Linux** (x86_64; glibc 2.35 or newer, e.g. Ubuntu 22.04+, Debian 12+, Fedora 36+)

| File | What it is |
|---|---|
| `cloudnode-<ver>-linux-x86_64.tar.gz` | `bin/cloudnode` and `bin/cloudnode-tui`, with the desktop entry and icons under `share/`. To install for your user: `tar -xzf cloudnode-…tar.gz --strip-components 1 -C ~/.local` |

## Build from source

You need a recent stable Rust (1.90 or newer). On Linux, install the libraries Slint needs first
(Debian/Ubuntu):

```sh
sudo apt install libfontconfig1-dev libfreetype-dev libxkbcommon-dev libwayland-dev \
  libx11-dev libxcursor-dev libxrandr-dev libxi-dev libgl1-mesa-dev
```

Then:

```sh
git clone https://github.com/TradeCloud/CloudNode
cd CloudNode
cargo run --release -p cloudnode        # the desktop app
cargo run --release -p cloudnode-tui    # the console app (q, Esc or Ctrl+C quits)
cargo xtask ci                          # fmt, clippy, tests and the layering check
packaging/package.sh                    # the release files for this system, into dist/
```

New contributors and AI agents: start with [`AGENTS.md`](AGENTS.md). Work happens on the
`develop` branch; every push to `main` builds a draft release for all three systems
([`docs/releasing.md`](docs/releasing.md)).

> [!IMPORTANT]
> **Status:** CloudNode is at the very start (pre-alpha). The workspace, the window and the
> console interface are in place, with empty lists; the vault, SSH sessions and the terminal
> come next. See [`ROADMAP.md`](ROADMAP.md), and please file issues with your OS and what you
> were doing.

## License and credits

CloudNode is dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your
option. Copyright (c) 2026 TradeCloud and the CloudNode contributors. Required notices are in
[NOTICE](NOTICE).

Bundled fonts, icons and other assets keep their own licenses; each one is listed with its
author, source and license in [ATTRIBUTION.md](ATTRIBUTION.md).

The desktop app is built with [Slint](https://slint.dev), used under the Slint Royalty-free
Desktop, Mobile, and Web Applications License 2.0.

**The TradeCloud name, logo and the CloudNode cheetah are not licensed under MIT or
Apache-2.0.** They (everything in [`docs/brand/`](docs/brand/), plus the app icon in
[`assets/app-icon/`](assets/app-icon/)) are trademarks of TradeCloud and may be used only
unmodified, and only as part of this repository and CloudNode, under
[`docs/brand/LICENSE-brand.txt`](docs/brand/LICENSE-brand.txt). Forks and modified versions must
remove them.

<p align="center">
  <img alt="" src="assets/app-icon/hicolor/64x64/apps/com.tradecloud.cloudnode.png" width="32"><br>
  <sub>Made by <b>TradeCloud</b>.</sub>
</p>
