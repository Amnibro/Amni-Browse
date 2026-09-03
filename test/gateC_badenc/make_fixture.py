import pathlib
p = pathlib.Path(__file__).parent
# UTF-8 BOM + "café" — encoding_rs may decode off x-user-defined; U+00E9 is outside 0xF780..0xF7FF.
(p / "favicon.ico").write_bytes(b"\xef\xbb\xbfcaf\xc3\xa9")
(p / "index.html").write_text(
    "<!doctype html><html><head><title>badenc</title></head><body>no link rel icon</body></html>",
    encoding="utf-8",
)
print(len((p / "favicon.ico").read_bytes()))
