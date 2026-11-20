@echo off
setlocal enabledelayedexpansion
cd /d "%~dp0"

REM Builds the app and the pvc command-line tool, after running every check.
REM
REM   build.bat            a test build: version 0.1.0-a1b2c3d, the .exe only
REM   build.bat --release  the real thing: version 0.1.0, with an installer
REM
REM The version lives in package.json. Keep src-tauri\tauri.conf.json and the
REM root Cargo.toml the same when you change it.

for /f "delims=" %%v in ('node -p "require('./package.json').version"') do set BASE_VERSION=%%v

if "%1"=="--release" (
    set FULL_VERSION=%BASE_VERSION%
    set BUNDLE_FLAGS=
    echo Building RELEASE version: %BASE_VERSION%
) else (
    for /f "delims=" %%h in ('git rev-parse --short HEAD') do set COMMIT_HASH=%%h
    set FULL_VERSION=%BASE_VERSION%-!COMMIT_HASH!
    set BUNDLE_FLAGS=--no-bundle
    echo Building TEST version: !FULL_VERSION!
)

if not exist node_modules (
    echo === Fetching the window's building blocks ===
    call npm install --ignore-scripts || exit /b 1
)

echo === Checking the window ===
call npm run check || exit /b 1
call npm test || exit /b 1

echo === Checking the engine ===
cargo test -p pvc-engine -p pvc || exit /b 1
cargo check -p prayer-video-creator || exit /b 1

echo === Building the command-line tool ===
cargo build --release -p pvc || exit /b 1

echo === Building the app (v%FULL_VERSION%) ===
echo {"version":"%FULL_VERSION%"} > .tauri-build-config.json
call npm run tauri build -- --config .tauri-build-config.json %BUNDLE_FLAGS% || (del .tauri-build-config.json & exit /b 1)
del .tauri-build-config.json

echo.
echo === Done (v%FULL_VERSION%) ===
echo The app:               target\release\prayer-video-creator.exe
echo The command-line tool: target\release\pvc.exe
if "%1"=="--release" echo The installer:         target\release\bundle\
