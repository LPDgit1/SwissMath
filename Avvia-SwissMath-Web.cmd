@echo off
setlocal
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\serve-web.ps1"
if errorlevel 1 pause
