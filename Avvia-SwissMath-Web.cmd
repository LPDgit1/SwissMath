@echo off
setlocal
if exist "%~dp0release\SwissMath-Web-Portable.exe" (
  start "SwissMath Web" "%~dp0release\SwissMath-Web-Portable.exe"
  exit /b 0
)
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\serve-web.ps1"
if errorlevel 1 pause
