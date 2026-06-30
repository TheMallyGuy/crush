@echo off

net session >nul 2>&1
if %errorLevel% == 0 (
    crush.exe -l %1
) else (
    echo "This file is an helper to rerun crush, please accept the UAC prompt in order for Swifttunnel to work."
    powershell -Command "Start-Process 'crush.exe' -ArgumentList '-l %1' -Verb RunAs"
)