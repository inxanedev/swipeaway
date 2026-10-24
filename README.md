# SwipeAway

SwipeAway is a Windows utility for cleaning up a folder one item at a time. Point it at a
folder, preview each file or subfolder, then choose **Keep**, **Delete**, or **Manual**.

Nothing is permanently deleted. **Delete** moves the item into a `swipeaway-junk` folder
inside the folder you picked, and **Manual** moves it into `swipeaway-manual`. You review and
empty those folders yourself whenever you're ready.

## Features

- Previews images, text and code, video thumbnails/filmstrips, audio, fonts, ZIP contents,
  folders, and Windows-provided previews for formats such as PDF and Office documents.
- Keep / Delete / Manual actions, with undo (`Ctrl+Z`).
- **Show in folder** for every item.
- Browse folder contents read-only, and open that folder in a new SwipeAway window.
- Skips unsafe targets such as drive roots, the Windows directory, Program Files, ProgramData,
  and your entire user profile.
- Shows a warning for hidden and system items.
- Dark, keyboard-friendly UI. Settings and recent folders live in `%APPDATA%\SwipeAway`.

## Build

Requires Rust with the MSVC Windows toolchain.

```powershell
cargo run --release
```

The distributable is `target\release\swipeaway.exe`. The build is configured with the static
CRT, so the executable is self-contained. FFmpeg is optional: if it is on `PATH`, SwipeAway
uses it for extra video frames and audio waveforms; otherwise it falls back to Windows
thumbnails.

## Notes

Files are only ever moved. To finish cleaning up, review `swipeaway-junk` and
`swipeaway-manual` in Explorer and remove them when you're sure.

## License

MIT
