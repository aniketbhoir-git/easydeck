@echo off
rem Builds the A-1 focus spike (release) and starts it in run mode.
rem Extra arguments pass through, e.g.: run-spike.cmd --monitor 1 --topmost
setlocal
cd /d "%~dp0"
if exist "%USERPROFILE%\.cargo\bin" set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
cargo build --release
if errorlevel 1 (
  echo Build failed.
  pause
  exit /b 1
)
echo Starting the spike. Log: %TEMP%\easydeck-a1-spike.jsonl
start "" "target\release\easydeck-a1-focus-spike.exe" %*
