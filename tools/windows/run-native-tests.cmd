@echo off
setlocal
cd /d "%~dp0"
set "LAB_LOG=%TEMP%\ScrumeVPN-native-%RANDOM%-%RANDOM%.txt"
if exist "%LAB_LOG%" goto failed
echo UNSIGNED ARM64 LAB - DPAPI and console IPC, not a VPN or vault ACL test
echo [1/2] Launching service capabilities check...
ScrumeVPNSvc.exe --capabilities > "%LAB_LOG%" 2>&1
if errorlevel 1 goto showfailed
echo [2/2] Running 4 native tests. Output is being saved to:
echo "%LAB_LOG%"
windows-adapter-tests.exe --include-ignored --test-threads=1 >> "%LAB_LOG%" 2>&1
if errorlevel 1 goto showfailed
type "%LAB_LOG%"
echo.
echo NATIVE ADAPTER TESTS PASSED
echo Evidence: "%LAB_LOG%"
pause
exit /b 0
:showfailed
type "%LAB_LOG%"
:failed
echo NATIVE LAB TEST FAILED - preserve this window and log for inspection
pause
exit /b 1
