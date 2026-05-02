# clip-cleaner

A fast CLI tool for cleaning up old game clips to free up disk space.

## What it does

Scans a folder (recursively) for `.mp4` files, sorts them oldest-first, and deletes clips until the total folder size is under your target. Clips shorter than 2 minutes are never deleted.

## Usage

Run the executable and follow the prompts:

1. Enter the folder path to clean (e.g. `D:/AMD clipps`)
2. Enter your target size in GB
3. Confirm with `y`

## Requirements

- [ffmpeg/ffprobe](https://ffmpeg.org/download.html) must be installed and available in PATH

## Build

```bash
cargo build --release
```

## Notes

- Deletion is **permanent** — no recycle bin
- Clips are deleted oldest-first
- Clips under 2 minutes are skipped

(self note - time spend: 4h 20mins)
