# CloudNode app icon

A **running cheetah**, CloudNode's mark for speed: a hand-drawn ink drawing with TradeCloud
orange accents, on a light rounded tile.

**This is a TradeCloud trademark, not open source.** It is not covered by CloudNode's MIT OR
Apache-2.0 license; see [`LICENSE.txt`](LICENSE.txt) and
[`docs/brand/LICENSE-brand.txt`](../../docs/brand/LICENSE-brand.txt). Forks and modified versions
must remove it.

## Provenance

TradeCloud's original drawing, 2000 × 2000 px, full bleed, with transparent corners outside the
rounded tile. The master (PNG and layered PSD) is kept by TradeCloud and is not in this
repository.

## Files

All of them are rendered from the master by [`packaging/icons.py`](../../packaging/icons.py);
never edit them by hand:

```sh
python3 packaging/icons.py <path to the master PNG>
```

| File | Used for |
|---|---|
| `cloudnode-1024.png` | The full tile at 1024 px: README, store listings |
| `cloudnode-macos-1024.png` | The tile on Apple's icon grid (824 of 1024 px, centred) |
| `cloudnode.icns` | The macOS bundle icon (`CFBundleIconFile`), from the grid render |
| `cloudnode.ico` | The Windows icon, 16–256 px |
| `hicolor/<n>x<n>/apps/com.tradecloud.cloudnode.png` | The Linux icon theme, 16–512 px; the 256 px one is also the window icon (`crates/ui-slint/ui/app.slint`) |
