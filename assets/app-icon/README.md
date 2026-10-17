# PhotoHouse app icon

The **PhotoHouse paint roller**: a green roller with two eyes on a mint rounded square, the
roller's ends and handle running past the square's edges.

## Palette

| Colour | Hex | Used for |
|---|---|---|
| Mint | `#ddfde6` | the rounded square behind the roller |
| Greens | `#7add64` → `#13ae7c`, `#70d068` → `#0b6d67` | roller, handle and grip (gradients) |
| White | `#fdfdfd` | the eyes |

## Geometry

A 512-unit tile (`viewBox="0 0 512 512"`). The original artwork (`2264.77 x 1820.76`, not square)
is scaled uniformly, without stretching, and centred with a 32-unit margin on each side; the
outside of the artwork is transparent. The tile keeps the `rx=112` clip, which cuts nothing. The
macOS renders pad it onto Apple's 824/1024 icon grid; the Windows and Linux renders crop 22 units
off each side so the artwork reads at 16-48 px.

## Files

- `photocraft.svg`: canonical master.
- `photocraft-small.svg`: same drawing (the artwork is already light); also the hicolor scalable icon.
- `photocraft-1024.png`: 1024 px render on the macOS grid.
- `photocraft.icns`: macOS bundle icon (`CFBundleIconFile`).
- `photocraft.ico`: Windows icon, 16-256 px, embedded in the `.exe` by `apps/photocraft/build.rs`.
  `packaging/windows/photohouse.ico` (the PhotoHouse installer) is a copy of it.
- `hicolor/<size>/apps/ai.storyteller.photocraft.png` (16-512) and `hicolor/scalable/...svg`:
  Linux icon theme.

The app also sets the window icon and Wayland app ID at runtime (`apps/photocraft/src/app_icon.rs`,
`main.rs`). On Windows the taskbar shows the icon of the Start Menu shortcut that launches the
exe, so the MSI's `<Icon Id>` keeps the `.exe` extension (ICE50); `app_icon.rs` documents the
details and tests the `.ico` sizes and the WiX icon references.

## Regenerate

Replace `photocraft.svg` (and `photocraft-small.svg`), then run `packaging/icons.sh`. It needs
`resvg`, plus `iconutil` on macOS; the `.ico` is packed by `cargo xtask ico`. Then copy
`photocraft.ico` over `packaging/windows/photohouse.ico`.
