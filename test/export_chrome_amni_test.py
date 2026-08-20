import json, sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import export_chrome_amni as ex
def test_parse_bookmarks():
    raw = json.dumps({"roots": {"bookmark_bar": {"name": "Bookmarks bar", "type": "folder", "children": [{"type": "url", "name": "Ex", "url": "https://example.com/", "date_added": "13300000000000000"}]}}})
    rows = ex.parse_bookmarks(raw)
    assert rows[0]["url"] == "https://example.com/"
    assert rows[0]["title"] == "Ex"
def test_validate_rejects_password():
    try:
        ex.validate_payload({"version": 1, "password": "x"})
        raise SystemExit("should reject")
    except SystemExit:
        pass
if __name__ == "__main__":
    test_parse_bookmarks()
    test_validate_rejects_password()
    print("ok")
