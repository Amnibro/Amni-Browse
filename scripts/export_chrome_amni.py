#!/usr/bin/env python3
import json, os, shutil, sqlite3, sys, tempfile
from datetime import datetime, timezone
from pathlib import Path
CHROME_EPOCH_MS = 11644473600000
CAP = 50000
BANNED = ("password", "cookie", "token")
def chrome_time_to_unix_ms(v):
    try:
        n = int(v)
    except (TypeError, ValueError):
        return 0
    if n <= 0:
        return 0
    return max(0, n // 1000 - CHROME_EPOCH_MS)
def walk_bm(node, path, out):
    t = node.get("type") or ""
    if t == "url":
        url = node.get("url") or ""
        if url.startswith("http"):
            title = node.get("name") or url
            added = chrome_time_to_unix_ms(node.get("date_added") or 0)
            out.append({"title": title, "url": url, "path": path[:], "added": added})
        return
    name = node.get("name")
    kids = node.get("children") or []
    nxt = path + ([name] if name else [])
    for k in kids:
        if isinstance(k, dict):
            walk_bm(k, nxt, out)
def parse_bookmarks(text):
    data = json.loads(text)
    out = []
    roots = data.get("roots") or {}
    for key, node in roots.items():
        if not isinstance(node, dict):
            continue
        label = node.get("name") or key
        walk_bm(node, [label], out)
    return out
def parse_history(db_path):
    con = sqlite3.connect(f"file:{db_path}?mode=ro", uri=True)
    con.row_factory = sqlite3.Row
    rows = con.execute(
        "SELECT url, title, last_visit_time, visit_count FROM urls ORDER BY last_visit_time DESC LIMIT ?",
        (CAP,),
    ).fetchall()
    con.close()
    hist = []
    for r in rows:
        url = r["url"] or ""
        if not url.startswith("http"):
            continue
        hist.append(
            {
                "url": url,
                "title": r["title"] or url,
                "lastVisit": chrome_time_to_unix_ms(r["last_visit_time"]),
                "visitCount": int(r["visit_count"] or 0),
            }
        )
    return hist
def validate_payload(obj):
    blob = json.dumps(obj)
    low = blob.lower()
    for b in BANNED:
        if f'"{b}"' in low:
            raise SystemExit(f"refusing payload with banned key {b}")
def export(profile: Path, dest: Path):
    bm_path = profile / "Bookmarks"
    hist_path = profile / "History"
    bookmarks = []
    history = []
    notes = []
    if bm_path.exists():
        bookmarks = parse_bookmarks(bm_path.read_text(encoding="utf-8"))
    else:
        notes.append("bookmarks missing")
    if hist_path.exists():
        fd, tmp = tempfile.mkstemp(suffix=".sqlite")
        os.close(fd)
        try:
            shutil.copy2(hist_path, tmp)
            history = parse_history(tmp)
        except Exception as e:
            notes.append(f"history failed: {e}")
        finally:
            try:
                os.remove(tmp)
            except OSError:
                pass
    else:
        notes.append("history missing")
    payload = {
        "version": 1,
        "source": "chrome-windows",
        "profile": profile.name,
        "exportedAt": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "bookmarks": bookmarks,
        "history": history,
        "notes": notes,
    }
    validate_payload(payload)
    dest.parent.mkdir(parents=True, exist_ok=True)
    dest.write_text(json.dumps(payload, ensure_ascii=False), encoding="utf-8")
    print(f"wrote {dest} bookmarks={len(bookmarks)} history={len(history)}")
def main():
    local = Path(os.environ.get("LOCALAPPDATA", ""))
    profile = Path(sys.argv[1]) if len(sys.argv) > 1 else local / "Google" / "Chrome" / "User Data" / "Default"
    dest = Path(sys.argv[2]) if len(sys.argv) > 2 else Path.home() / "Documents" / "amni-chrome-import.json"
    export(profile, dest)
if __name__ == "__main__":
    main()
