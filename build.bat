@echo off
setlocal
chcp 65001 >nul
title NHK4 - Build Manager

cd /d "%~dp0"

echo ========================================================
echo  NHK4 - Build Manager
echo ========================================================
echo.
echo  [1] Debug Build   - Fast compile for testing
echo  [2] Release Build - Optimized binary for production
echo.
set /p build_choice="Select mode [1 or 2, default 1]: "

if "%build_choice%"=="2" goto build_release

:build_debug
echo.
echo Compiling Debug build...
cargo build
if %ERRORLEVEL% EQU 0 (
    echo.
    echo ========================================================
    echo  [SUCCESS] Debug build finished!
    echo  Binary: target\debug\nhk4.exe
    echo ========================================================
) else (
    echo.
    echo ========================================================
    echo  [FAILED] Debug build failed.
    echo ========================================================
)
goto end

:build_release
echo.
echo Compiling Release build...
cargo build --release
if %ERRORLEVEL% EQU 0 (
    echo.
    echo ========================================================
    echo  [SUCCESS] Release build finished!
    echo  Binary: target\release\nhk4.exe
    echo ========================================================
) else (
    echo.
    echo ========================================================
    echo  [FAILED] Release build failed.
    echo ========================================================
)
goto end

:end
echo.
pause
