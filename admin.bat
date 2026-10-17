@echo off
setlocal
cd /d "%~dp0\admin-web"
title NHK License Portal (Next.js Admin)

echo ===================================================
echo   NHK License Portal - Next.js Admin Management
echo ===================================================
echo Starting Admin Dev Server on http://localhost:3000
echo Press Ctrl+C to stop.
echo.

start "" "http://localhost:3000"
npm run dev

pause
