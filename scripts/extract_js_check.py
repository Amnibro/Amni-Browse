from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
src = Path(ROOT / "src" / "ui" / "webview.rs").read_text(encoding="utf-8")
start = src.find("<script>")
end = src.find("</script>", start)
if start == -1 or end == -1:
    raise SystemExit("script tags not found")

js = src[start + 8:end]
js = js.replace("{{", "{").replace("}}", "}")
js = re.sub(r"\{e_[^}]+\}", '"x"', js)
js = js.replace("{css_vars}", "")

out = ROOT / "target" / "_ui_script_check.js"
out.parent.mkdir(parents=True, exist_ok=True)
out.write_text(js, encoding="utf-8")
print(f"wrote {len(js)} bytes to {out}")
