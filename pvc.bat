@echo off
REM Runs the pvc command-line tool from this folder, building it first if needed.
REM
REM   pvc.bat list
REM   pvc.bat show luminous
REM   pvc.bat make luminous
REM
REM Run "pvc.bat help" to see everything.

cargo run --quiet --release --manifest-path "%~dp0Cargo.toml" -p pvc -- %*
