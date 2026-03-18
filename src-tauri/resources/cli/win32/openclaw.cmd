@echo off
setlocal

if /i "%1"=="update" (
  echo openclaw is managed by ClawPet ^(bundled runtime^).
  exit /b 0
)

set "SCRIPT_DIR=%~dp0"
set "ENTRY="
if exist "%SCRIPT_DIR%..\openclaw\openclaw.mjs" set "ENTRY=%SCRIPT_DIR%..\openclaw\openclaw.mjs"
if not defined ENTRY if exist "%SCRIPT_DIR%..\..\openclaw\openclaw.mjs" set "ENTRY=%SCRIPT_DIR%..\..\openclaw\openclaw.mjs"
if not defined ENTRY if exist "%SCRIPT_DIR%..\resources\openclaw\openclaw.mjs" set "ENTRY=%SCRIPT_DIR%..\resources\openclaw\openclaw.mjs"

if not defined ENTRY (
  echo Error: bundled openclaw.mjs not found.
  exit /b 1
)

set "NODE_BIN="
if defined OPENCLAW_NODE_PATH if exist "%OPENCLAW_NODE_PATH%" set "NODE_BIN=%OPENCLAW_NODE_PATH%"
if not defined NODE_BIN if exist "%SCRIPT_DIR%..\bin\node.exe" set "NODE_BIN=%SCRIPT_DIR%..\bin\node.exe"
if not defined NODE_BIN if exist "%SCRIPT_DIR%..\resources\bin\node.exe" set "NODE_BIN=%SCRIPT_DIR%..\resources\bin\node.exe"
if not defined NODE_BIN where node >nul 2>nul && set "NODE_BIN=node"

if not defined NODE_BIN (
  echo Error: Node.js binary not found. Set OPENCLAW_NODE_PATH or install node.
  exit /b 1
)

set OPENCLAW_EMBEDDED_IN=ClawPet
set OPENCLAW_NO_RESPAWN=1
"%NODE_BIN%" "%ENTRY%" %*
set _EXIT=%ERRORLEVEL%
endlocal & exit /b %_EXIT%
