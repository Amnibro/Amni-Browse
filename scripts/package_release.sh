#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
VER=$(grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)"/\1/')
EXE=target/release/amni-browse.exe
BASEZIP=$(ls amni-browse-v*-win64.zip | sort -V | tail -1)
OUT=amni-browse-v${VER}-win64.zip
grep -q "Real Servo (libservo)" "$EXE" || { echo "FATAL: exe is not a servo-real build"; exit 1; }
grep -q "WebView (wry/tao)" "$EXE" && { echo "FATAL: exe is a webview build"; exit 1; }
STAGE=$(mktemp -d)
unzip -q "$BASEZIP" -d "$STAGE"
cp "$EXE" "$STAGE/amni-browse.exe"
rm -rf "$STAGE/assets"
cp -r assets "$STAGE/assets"
rm -f "$STAGE/assets/windows_app.rc" "$STAGE"/assets/*.jpg "$STAGE"/assets/grok-image-*.png
python - "$STAGE" "$OUT" <<'EOF'
import os,sys,zipfile
stage,out=sys.argv[1],sys.argv[2]
z=zipfile.ZipFile(out,'w',zipfile.ZIP_DEFLATED)
for root,_,files in os.walk(stage):
    for f in files:
        p=os.path.join(root,f)
        z.write(p,os.path.relpath(p,stage).replace(os.sep,'/'))
z.close()
EOF
rm -rf "$STAGE"
unzip -l "$OUT" | tail -2
echo "packed $OUT from base $BASEZIP with servo-real exe $(stat -c%s "$EXE") bytes"
