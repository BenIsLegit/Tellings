@echo off
cd /d "%~dp0"

REM Opens the app straight from this folder, for trying changes.
REM The first time takes a few minutes while everything is built.

if not exist node_modules (
    echo Fetching the window's building blocks. This happens once.
    call npm install --ignore-scripts || exit /b 1
)
call npm run tauri dev
