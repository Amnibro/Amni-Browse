@echo off
setlocal
cd /d "%~dp0\.."
echo Publishing Amni-Browse v0.12.6 release asset...
gh release create v0.12.6 "target\release\amni-browse-v0.12.6-win64.zip" --repo Amnibro/Amni-Browse --title "v0.12.6 — tab sync, sticky titles, less flicker" --notes "Tab visibility sync, sticky tab titles, less strip flicker. Install from https://amni-scient.com/amni-browse.html or this zip."
if errorlevel 1 (
  echo If the release already exists, uploading the asset instead...
  gh release upload v0.12.6 "target\release\amni-browse-v0.12.6-win64.zip" --repo Amnibro/Amni-Browse --clobber
)
echo.
echo Merging site PR so Pages serves latest.json...
gh pr merge 1 --repo Amnibro/Amni-Scient --merge --delete-branch
echo.
echo Done. Feed should point at:
echo https://github.com/Amnibro/Amni-Browse/releases/download/v0.12.6/amni-browse-v0.12.6-win64.zip
endlocal
