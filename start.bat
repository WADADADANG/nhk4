@echo off
setlocal
title NodeHotKeyV4 - Launcher

cd /d "%~dp0"

echo ========================================================
echo  NodeHotKeyV4 - Checking updates and launching...
echo ========================================================
rem Terminate previous instance if running to prevent file lock
taskkill /f /im nodehotkey-rust.exe >nul 2>&1

cargo build -q
if %ERRORLEVEL% NEQ 0 goto build_error

start "" "target\debug\nodehotkey-rust.exe"
exit /b

:build_error
echo.
echo ========================================================
echo  [ERROR] Build failed. Please inspect errors above.
echo ========================================================
echo.
pause
