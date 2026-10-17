@echo off
setlocal
title NHK4 - Launcher

cd /d "%~dp0"

echo ========================================================
echo  NHK4 - Checking updates and launching...
echo ========================================================
rem Terminate previous instance if running to prevent file lock
taskkill /f /im nhk4.exe >nul 2>&1
taskkill /f /im nodehotkey-rust.exe >nul 2>&1

cargo build -q
if %ERRORLEVEL% NEQ 0 goto build_error

start "" "target\debug\nhk4.exe"
exit /b

:build_error
echo.
echo ========================================================
echo  [ERROR] Build failed. Please inspect errors above.
echo ========================================================
echo.
pause
