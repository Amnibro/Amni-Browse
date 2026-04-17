from pathlib import Path
import re

src = Path(r"C:\Users\antho\Documents\ai\Amni-Browse\src\ui\webview.rs").read_text(encoding="utf-8")
start = src.find("<script>")
end = src.find("</script>", start)
if start == -1 or end == -1:
    raise SystemExit("script tags not found")

js = src[start + 8:end]
js = js.replace("{{", "{").replace("}}", "}")
js = re.sub(r"\{e_[^}]+\}", '"x"', js)
js = js.replace("{css_vars}", "")

out = Path(r"C:\Users\antho\Documents\ai\Amni-Browse\target\_ui_script_check.js")
out.write_text(js, encoding="utf-8")
print(f"wrote {len(js)} bytes to {out}")
