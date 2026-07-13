@echo off

where nasm >nul 2>&1
if not %errorlevel%==0 (
    echo [WARN] NASM not found in PATH. Rust may fail if it requires nasm.
    echo        Install NASM: https://www.nasm.us
    echo.
)

bun tauri build
if not %errorlevel%==0 (
    echo.
    echo [FAIL] Build failed.
    pause
    goto :eof
)

echo.
echo [OK] Copying exe to bin\...
if not exist bin mkdir bin
copy /y "src-tauri\target\i686-pc-windows-msvc\release\nadeloader.exe" bin\ >nul
echo [OK] Done - bin\nadeloader.exe
pause
