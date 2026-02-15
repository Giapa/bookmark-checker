# bookmark-checker

A CLI tool that cleans up HTML bookmark files exported from browsers. It removes duplicate bookmarks and checks for outdated URLs that no longer resolve.

## Installation

### Download a pre-built binary

Grab the latest binary for your platform from the [Releases](../../releases/latest) page:

| Platform | File |
|---|---|
| Linux (x86_64) | `bookmark-checker-linux-x86_64` |
| macOS (Intel) | `bookmark-checker-macos-x86_64` |
| macOS (Apple Silicon) | `bookmark-checker-macos-aarch64` |
| Windows (x86_64) | `bookmark-checker-windows-x86_64.exe` |

On Linux/macOS, make it executable after downloading:

```sh
chmod +x bookmark-checker-*
```

### Build from source

Requires [Rust](https://www.rust-lang.org/tools/install) (1.85+):

```sh
cargo build --release
```

The binary will be at `target/release/bookmark-checker`.

## Usage

```sh
bookmark-checker <FILE_PATH> [-o <OUTPUT>]
```

### Arguments

| Argument | Description |
|---|---|
| `FILE_PATH` | Path to the input HTML bookmarks file |
| `-o`, `--output` | Output file path (optional, defaults to `<input>-cleaned.html`) |

### Examples

Process a bookmarks file (writes to `bookmarks-cleaned.html`):

```sh
cargo run -- ./bookmarks.html
```

Specify a custom output path:

```sh
cargo run -- ./bookmarks.html -o cleaned.html
```

## What it does

1. Parses the HTML bookmarks file
2. Finds and removes duplicate bookmark entries
3. Checks each URL to see if it still resolves
4. Removes bookmarks with broken/outdated URLs
5. Writes the cleaned result to the output file
