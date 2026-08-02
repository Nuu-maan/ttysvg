# ttysvg

Record your terminal, get one animated SVG you can drop straight into a README.

<p align="center">
  <img src="docs/demo.svg" alt="ttysvg recording itself" width="100%">
</p>

That image is a single file. No JavaScript, no video, no external requests. The text
in it is real text, so it stays sharp at any zoom and you can select and copy it. It
also carries a light palette and a dark palette at once, so it matches whichever theme
the reader is using.

It works on Windows, which is the reason this project exists.

## What you get

- **One file.** A `.svg` you commit next to your code. GitHub renders it inline.
- **Small.** A typical recording is 20 to 100 KB. The same thing as a GIF is several megabytes.
- **Sharp forever.** Vector text, not pixels. Zoom in as far as you like.
- **Theme aware.** One file looks correct on both the light and dark versions of a page.
- **Repeatable.** Write your demo as a short script and regenerate it whenever your output changes.

## Why this exists

A command line tool is judged by the demo at the top of its README. If you develop on
Windows, there was no good way to make one.

| Tool | Why it does not work on Windows |
|---|---|
| asciinema | Unix only. There is no Windows recorder, because it depends on a Unix pty. |
| VHS | Requires ttyd and ffmpeg. The Windows path is unofficial and breaks easily. |
| svg-term-cli | Unmaintained, and it needs an asciinema recording as its input. |
| Screen capture to GIF | Multi megabyte, blurry when scaled, wrong colors in dark mode, and a single typo means recording the whole thing again. |

`ttysvg` talks to the Windows pseudo console (ConPTY) directly. The same code runs on
macOS and Linux through a normal pty, so a project can use one tool everywhere.

## Install

You need [Rust](https://rustup.rs). On Windows you also need Windows 10 version 1809 or
newer, which is when ConPTY arrived.

```
git clone https://github.com/Nuu-maan/ttysvg
cd ttysvg
cargo install --path .
```

That puts `ttysvg` on your PATH. To try it without installing, use `cargo run --` in
place of `ttysvg` in any command below.

## Your first recording

Run a program and record it. Everything works normally while it runs, and the file is
written when the program exits.

```
ttysvg record --out demo.svg -- cargo build
```

Recording an interactive shell works the same way. Type whatever you want, then `exit`.

```
ttysvg record --out demo.svg -- powershell -NoLogo
```

Open `demo.svg` in a browser to check it, then commit it and reference it from your
README:

```markdown
<img src="demo.svg" alt="demo" width="100%">
```

## Making it repeatable

Recording by hand is fine once. The problem comes later, when your output changes and
every demo in the repo is out of date.

A tape fixes that. It is a small script describing the demo, so you can regenerate the
recording any time with one command.

```
ttysvg build demo.tape
```

Here is a complete tape with every line explained:

```tape
output "demo.svg"           # where to write the result
theme  "tokyonight"         # built in theme name, or a path to your own
width  80                   # recording size in characters, not pixels
height 20
window on                   # draw a title bar with traffic lights
title  "demo"

shell "powershell.exe" "-NoLogo" "-NoProfile"

type-delay 55ms             # pause between keystrokes, so typing looks human
trim-idle  700ms            # cut any dead air longer than this
tail       2s               # hold the last frame this long before looping

wait  "PS " 20s             # block until the prompt appears, up to 20 seconds
type  "cargo build"         # type it out, one character at a time
sleep 400ms
enter
wait  "Finished" 60s        # block until the build actually finishes
sleep 1s
```

`wait` is the part that matters most. It blocks until that text really appears on the
screen. A fixed `sleep` guesses, and the guess is wrong the first time the machine is
busy, which is how scripted demos end up cut off halfway. `wait` cannot desynchronize.

Once you have a tape, rebuilding in a different theme costs nothing:

```
ttysvg build demo.tape --theme gruvbox --out demo-gruvbox.svg
```

## Recording once and restyling forever

Rebuilding a tape re-runs your command, which takes as long as the command takes and
depends on the machine still being in the same state. If all you want to change is how
the recording looks, save the capture instead:

```
ttysvg record --out demo.svg --save demo.json -- cargo build
```

`demo.json` holds the terminal frames and their timing, not the SVG. Every style and
timing choice can then be replayed against it with no terminal involved:

```
ttysvg render demo.json --theme gruvbox
ttysvg render demo.json --speed 2 --window --title "cargo build"
ttysvg render demo.json --info
```

This is the difference between a two second loop and a two minute one. Rendering reads
the saved frames straight from disk, so it finishes in milliseconds however long the
original command took. Re-rendering with no flags reproduces the original SVG byte for
byte, because the recording stored the settings it was made with.

Both other commands can save one, and `--save` on its own skips the SVG entirely:

```
ttysvg record --save session.json -- ./my-tool
ttysvg build demo.tape --save demo.json
```

## Command reference

### ttysvg record

Captures a live session. Stops when the program exits.

| Flag | Default | Meaning |
|---|---|---|
| `--out` | `demo.svg` | Output path |
| `--save` | off | Also write the raw capture, for `ttysvg render` |
| `--cols` `--rows` | your terminal size | Recording size in characters |
| `--theme` | `tokyonight` | Theme name or path to a theme file |
| `--font` | system monospace stack | CSS font family used in the output |
| `--font-size` | `14` | Pixels |
| `--padding` | `18` | Pixels of space around the content |
| `--window` | off | Draw a title bar |
| `--title` | empty | Text shown in the title bar |
| `--trim-idle` | `1s` | Collapse pauses longer than this, or `off` to keep real timing |
| `--speed` | `1.0` | Playback multiplier, so `2` is twice as fast |
| `--tail` | `2s` | How long the final frame holds before the loop restarts |
| `--no-loop` | off | Play once instead of looping |
| `--advance` | auto | Override character width in pixels, see Known limits |
| `--line-height` | auto | Override line height in pixels |

Everything after `--` is the command to record.

### ttysvg build

Runs a tape. Any flag given here overrides the tape.

| Flag | Meaning |
|---|---|
| `--out` | Output path |
| `--save` | Also write the raw capture, for `ttysvg render` |
| `--theme` | Theme name or path |
| `--speed` | Playback multiplier |
| `--window` | Force the title bar on |
| `--title` | Title bar text |

### ttysvg render

Rebuilds an SVG from a saved capture. Nothing is executed. Anything not given keeps the
value the recording was made with.

| Flag | Default | Meaning |
|---|---|---|
| `--out` | the capture path with an `.svg` extension | Output path |
| `--info` | off | Print what the capture holds and exit |
| `--theme` `--font` `--font-size` | as recorded | Same meaning as on `record` |
| `--padding` `--advance` `--line-height` | as recorded | Same meaning as on `record` |
| `--trim-idle` `--speed` `--tail` | as recorded | Retime without re-running anything |
| `--window` `--no-window` `--title` | as recorded | Title bar, either direction |
| `--no-loop` | as recorded | Play once instead of looping |

### ttysvg themes

Lists the built in themes.

### Tape directives

Settings, valid anywhere in the file:

| Directive | Example |
|---|---|
| `output` | `output "docs/demo.svg"` |
| `theme` | `theme "catppuccin"` |
| `width` `height` | `width 90` |
| `font` | `font "JetBrains Mono" 14` |
| `font-size` `padding` | `padding 20` |
| `advance` `line-height` | `advance 8.4` |
| `shell` | `shell "bash" "-i"` |
| `window` `title` `loop` | `window on` |
| `trim-idle` `tail` `speed` | `trim-idle 500ms` |
| `type-delay` | `type-delay 40ms` |

Actions, run in order:

| Directive | Example |
|---|---|
| `type` | `type "cargo build"` |
| `wait` | `wait "Finished" 30s` |
| `sleep` | `sleep 1.5s` |
| `enter` `tab` `backspace` `escape` `space` | `enter` |
| `up` `down` `left` `right` | `down 3` |
| `ctrl` | `ctrl c` |

Key directives take an optional repeat count, so `down 3` presses it three times.
Durations accept `ms`, `s` and `m`, and a bare number means milliseconds. Lines
starting with `#` are comments. Backslashes inside quotes are literal, so Windows
paths like `type ".\scripts\build.ps1"` work as written.

## Themes

```
ttysvg themes
```

Ships with `tokyonight`, `catppuccin`, `gruvbox` and `nord`. Each one defines a light
and a dark palette, and the SVG switches between them using `prefers-color-scheme`.

To use your own, write a TOML file and pass its path to `--theme`:

```toml
name = "mine"

[dark]
fg = "#c0caf5"
bg = "#1a1b26"
ansi = ["#15161e", "#f7768e", "#9ece6a", "#e0af68", "#7aa2f7", "#bb9af7", "#7dcfff", "#a9b1d6",
        "#414868", "#f7768e", "#9ece6a", "#e0af68", "#7aa2f7", "#bb9af7", "#7dcfff", "#c0caf5"]

[light]
fg = "#3760bf"
bg = "#e1e2e7"
ansi = ["#b4b5b9", "#f52a65", "#587539", "#8c6c3e", "#2e7de9", "#9854f1", "#007197", "#6172b0",
        "#a1a6c5", "#f52a65", "#587539", "#8c6c3e", "#2e7de9", "#9854f1", "#007197", "#3760bf"]
```

The 16 entries are the standard ANSI colors in order: black, red, green, yellow, blue,
magenta, cyan, white, then the same eight again in their bright forms.

## How it works

```mermaid
flowchart LR
  A["ConPTY / Unix pty<br/>portable-pty"] -->|bytes + timestamps| B["VT parser<br/>vt100"]
  B -->|screen snapshots| C["frame timeline"]
  C --> D["optimizer<br/>dedupe, trim idle, quantize"]
  D --> E["SVG emitter<br/>one @keyframes"]
  F[".tape script"] -->|drives input, waits on screen state| A
```

Escape sequences are never turned into SVG directly. The byte stream feeds a real
terminal grid that understands cursor movement, scroll regions, the alternate screen and
line wrapping, and each state of that grid is photographed into a frame. That is the
difference between recording a full screen TUI correctly and only recording `echo`
correctly.

The output stacks every frame vertically inside one clipped group and animates
`translateY` using a single `@keyframes` block with `step-end` timing. One CSS rule
drives the whole animation no matter how many frames there are, and frames of different
durations come for free.

Everything after the parser is platform independent, which is why the emitter is tested
against fixed byte streams with no terminal involved at all.

## Contributing

Contributions are genuinely welcome, including from people who have never written Rust
before. The codebase is small on purpose.

```
git clone https://github.com/Nuu-maan/ttysvg
cd ttysvg
cargo test
cargo run -- record --out demo.svg -- powershell -NoLogo
```

Where things live:

```
src/capture/    spawning a pty, reading it, running a tape against it
src/term/       terminal grid to Frame, the platform independent boundary
src/optimize.rs dedupe, trim idle, quantize, speed, tail
src/session.rs  saving and loading a capture, so it can be re-rendered
src/svg/        the emitter, themes, escaping
src/tape/       tape tokenizer and directives
themes/         one TOML file per theme
tests/          integration tests, including a snapshot of the emitter output
```

Good places to start, roughly easiest first:

- **Add a theme.** Copy a file in `themes/`, change the colors, add one line to the list in `src/svg/theme.rs`. No Rust knowledge needed beyond editing an array.
- **Add a tape directive.** One arm in the match in `src/tape/parse.rs`, plus a test.
- **Measured font metrics.** Right now character width is assumed rather than measured. This is the most valuable open problem in the project, see Known limits.
- **PNG or GIF export** via `resvg`, for places that still refuse SVG.
- **A GitHub Action** that runs `ttysvg build` on every tag so demos never go stale.

Before opening a pull request, run:

```
cargo test
cargo clippy --all-targets
cargo fmt
```

If you change the emitter, the snapshot test will fail on purpose. Review the diff, and
if the new output is correct, accept it with `cargo insta accept` or by setting
`INSTA_UPDATE=always`.

The code leans on naming and structure rather than comments. Please keep new code in the
same spirit, and put the explanation in the README or the pull request description where
people will actually read it.

Bug reports are just as useful as code. If a recording comes out wrong, attach the tape
and the resulting SVG, and say which terminal and Windows version you are on.

## Known limits

- **A saved capture cannot be resized.** `ttysvg render` can change every visual and
  timing setting except `--cols` and `--rows`. A capture stores the terminal grid after
  wrapping, not the bytes that produced it, so reflowing to a new width would need the
  recording to be made again.
- **Character width is assumed, not measured.** The output uses a generic monospace font
  stack and assumes each character is 0.6 times the font size wide. If text drifts to the
  right across a long line, pin it with `--advance` or the `advance` tape directive.
  Embedding a subset font would fix this properly and is the top item on the roadmap.
- **Wide characters depend on the viewer's font.** CJK, emoji and box drawing are placed
  using the terminal grid's own width tracking, but a font that disagrees will still look
  off by a cell.
- **Long recordings make large files.** Heavy animation means many frames. `trim-idle` and
  `speed` are the levers, and dropping the frame rate of the recorded program helps more
  than either.

## License

MIT. See [LICENSE](LICENSE).
