# Amni Browse in Helium (spike)

One Helium window showing CEF pages rendered off screen, with keyboard, mouse, wheel and tab input forwarded to CEF. `cef.gold` builds `cef.he` (the CEF module) and `main.gold` builds `main.he` (the window, input and tabs).

## Build

Requires Helium main at `df978100` or later (`c_callback`, `c_call` and `-DGS_OS_STACK`) with the gfx `PRESENT_ON_WORK` flag.

```
gold pack browse helium main=main.he
cd helium && gold build cef.gold && gold build main.gold
hc build main.he -o amni-browse-he -DGS_OS_STACK=1
```

`-DGS_OS_STACK=1` is required. Without it, V8 in the renderer processes, which run this same binary, stops at the first script on every page.

## Run

Use the CEF build that Amni Browse ships in `/usr/lib/amni-browse`. It supports Widevine; the CEF in `~/.cache/hc/cef-152-linux64` does not.

```
ln -s /usr/lib/amni-browse/v8_context_snapshot.bin /usr/lib/amni-browse/icudtl.dat .
HELIUM_CEF_DIR=/usr/lib/amni-browse LD_LIBRARY_PATH=/usr/lib/amni-browse ./amni-browse-he https://example.com https://wikipedia.org
```

- Each URL opens a tab.
- Shortcuts: Ctrl+T opens a tab, Ctrl+Tab and Ctrl+Shift+Tab switch tabs, Alt+Left and Alt+Right go back and forward, and F5 or Ctrl+R reloads.
- The profile is `~/.cache/amni-browse-helium`.
- `AB_STATS=1` prints, every second: frames drawn, CEF paints, key-to-paint time and present time.

## Measurements, 2026-11-25

Both builds ran on Xvfb with lavapipe (software Vulkan), on the same libcef from `/usr/lib/amni-browse`, with fresh profiles. The results still need confirming on a real GPU and display.

| Check | Helium spike | 0.16.4 |
|---|---|---|
| Memory, 1 tab (PSS, whole process tree) | 572–602 MB | 712 MB |
| Memory, 7 tabs | 1,110–1,386 MB | 1,735–1,756 MB |
| Widevine stream (Shaka, Angel One) | plays, 0 dropped | plays, 0 dropped |
| 1080p60 VP9, CPU decode | 47.8 fps shown, 3.4% dropped | 49.5 fps shown, 3.3% dropped |
| Wheel scrolling, page frame rate | 52–56 fps, worst gap 33–50 ms | 60.4–60.6 fps, worst gap 17 ms |
| Key to screen, median | 36–40 ms | 12 ms |

- **Main loop.** The loop presents without vsync (`NO_VSYNC | PRESENT_ON_WORK`), pumps CEF about every millisecond and redraws only when CEF paints. This cut key latency from 47 ms and video drops from 7.4%.
- **Where the key time goes.** About 10 ms is CEF's key-to-paint, and 5–20 ms is the software present.
- **YouTube.** Pages and video play.

## Next

- **Accelerated paint.** Use CEF's shared textures, imported into Vulkan as dma-bufs. This lifts the 60 fps cap of the software paint path and removes the copies and uploads.
- **External begin frames.** Drive them from the window so a frame starts right after input.
- **Real-display run.** Measure on a real GPU and display at the display's refresh rate.
