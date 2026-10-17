@echo off
setlocal
chcp 65001 >nul

echo ========================================================
echo  NodeHotKeyV4 - Running Unit Tests
echo ========================================================
echo.

cargo test -- --nocapture

if %ERRORLEVEL% EQU 0 (
    echo.
    echo ========================================================
    echo  [SUCCESS] All unit tests passed!
    echo ========================================================
) else (
    echo.
    echo ========================================================
    echo  [FAILED] Some tests failed. Check the logs above.
    echo ========================================================
)

echo.
pause
