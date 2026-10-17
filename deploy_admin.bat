@echo off
setlocal
chcp 65001 >nul
title NHK Admin Portal - Vercel Deployment

cd /d "%~dp0\admin-web"

echo ========================================================
echo  NHK Admin Portal - Vercel Deployer
echo ========================================================
echo.
echo  Current Directory: %CD%
echo.
echo  [1] Deploy to Production (npx vercel --prod)
echo  [2] Deploy Preview / First-time Setup (npx vercel)
echo  [3] Login to Vercel (npx vercel login)
echo.
set /p deploy_choice="Select option [1, 2, or 3, default 1]: "

if "%deploy_choice%"=="3" goto do_login
if "%deploy_choice%"=="2" goto do_preview

:do_prod
echo.
echo Deploying directly to Production...
call npx vercel --prod
goto finish

:do_preview
echo.
echo Running Vercel Setup / Preview Deployment...
call npx vercel
goto finish

:do_login
echo.
echo Logging in to Vercel...
call npx vercel login
goto finish

:finish
echo.
echo ========================================================
echo  Deployment process finished.
echo ========================================================
echo.
pause
