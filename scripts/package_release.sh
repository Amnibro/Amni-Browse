#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
VER=$(grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)"/\1/')
EXE=target/release/amni-browse.exe
OUT=amni-browse-v${VER}-win64.zip
grep -q "Chromium (WebView2)" "$EXE" || { echo "FATAL: exe is not a Chromium (WebView2) build"; exit 1; }
grep -q "Real Servo (libservo)" "$EXE" && { echo "FATAL: exe is a servo-real build; the shipped lane is Chromium (WebView2)"; exit 1; }
STAGE=$(mktemp -d)
mkdir -p "$STAGE/assets/chrome"
cp "$EXE" "$STAGE/amni-browse.exe"
cp assets/chrome/toolbar.html "$STAGE/assets/chrome/toolbar.html"
cp assets/amni-browse.ico "$STAGE/assets/" 2>/dev/null || true
cp LICENSE README.md CHANGELOG.md "$STAGE/"
printf 'Amni Browse %s (Windows x64, Chromium/WebView2 lane)\nRequires the Microsoft Edge WebView2 Runtime (bundled with Windows 10/11).\nRun amni-browse.exe, or use scripts/AmniBrowse-Setup.cmd for Start Menu + default-browser registration.\n' "$VER" > "$STAGE/INSTALL.txt"
python - "$STAGE" "$OUT" <<'PY'
import os,sys,zipfile
stage,out=sys.argv[1],sys.argv[2]
z=zipfile.ZipFile(out,'w',zipfile.ZIP_DEFLATED)
for root,_,files in os.walk(stage):
    for f in files:
        p=os.path.join(root,f)
        z.write(p,os.path.relpath(p,stage).replace(os.sep,'/'))
z.close()
PY
rm -rf "$STAGE"
cp "$OUT" amni-browse-win64.zip
unzip -l "$OUT" | tail -8
echo "packed $OUT (+ unversioned amni-browse-win64.zip) exe $(stat -c%s "$EXE") bytes"
