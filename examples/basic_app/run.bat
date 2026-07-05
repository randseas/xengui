@echo off
echo compiling...
cargo build
if %errorlevel% neq 0 (
    echo compile error!
    exit /b %errorlevel%
)
echo compile successful, starting app...
..\..\target\debug\basic_app.exe