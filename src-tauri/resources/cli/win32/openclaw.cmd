@echo off
setlocal EnableExtensions EnableDelayedExpansion

if /i "%1"=="update" (
  echo openclaw is managed by DragonClaw ^(bundled runtime^).
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
set "BEST_NODE_MAJOR="
set "BEST_NODE_MINOR="
set "BEST_NODE_PATCH="
if defined OPENCLAW_NODE_PATH if exist "%OPENCLAW_NODE_PATH%" (
  set "NODE_BIN=%OPENCLAW_NODE_PATH%"
) else (
  call :consider_node "%SCRIPT_DIR%..\bin\node.exe"
  call :consider_node "%SCRIPT_DIR%..\resources\bin\node.exe"
  for /f "delims=" %%N in ('where node 2^>nul') do call :consider_node "%%~fN"
)

if not defined NODE_BIN (
  echo Error: Node.js binary not found. Set OPENCLAW_NODE_PATH or install node.
  exit /b 1
)

set OPENCLAW_EMBEDDED_IN=DragonClaw
set OPENCLAW_NO_RESPAWN=1
"%NODE_BIN%" "%ENTRY%" %*
set _EXIT=%ERRORLEVEL%
endlocal & exit /b %_EXIT%

:consider_node
set "CANDIDATE=%~1"
if not defined CANDIDATE goto :eof
if not exist "%CANDIDATE%" goto :eof

set "CANDIDATE_VERSION="
for /f "delims=" %%V in ('"%CANDIDATE%" -v 2^>nul') do (
  if not defined CANDIDATE_VERSION set "CANDIDATE_VERSION=%%V"
)
if not defined CANDIDATE_VERSION goto :eof
set "CANDIDATE_VERSION=!CANDIDATE_VERSION:v=!"

set "CAND_MAJOR="
set "CAND_MINOR="
set "CAND_PATCH="
for /f "tokens=1-3 delims=." %%a in ("!CANDIDATE_VERSION!") do (
  set "CAND_MAJOR=%%a"
  set "CAND_MINOR=%%b"
  set "CAND_PATCH=%%c"
)
if not defined CAND_MAJOR set "CAND_MAJOR=0"
if not defined CAND_MINOR set "CAND_MINOR=0"
if not defined CAND_PATCH set "CAND_PATCH=0"

if not defined NODE_BIN (
  set "NODE_BIN=%CANDIDATE%"
  set "BEST_NODE_MAJOR=!CAND_MAJOR!"
  set "BEST_NODE_MINOR=!CAND_MINOR!"
  set "BEST_NODE_PATCH=!CAND_PATCH!"
  goto :eof
)

if !CAND_MAJOR! GTR !BEST_NODE_MAJOR! goto :select_candidate
if !CAND_MAJOR! LSS !BEST_NODE_MAJOR! goto :eof
if !CAND_MINOR! GTR !BEST_NODE_MINOR! goto :select_candidate
if !CAND_MINOR! LSS !BEST_NODE_MINOR! goto :eof
if !CAND_PATCH! GTR !BEST_NODE_PATCH! goto :select_candidate
goto :eof

:select_candidate
set "NODE_BIN=%CANDIDATE%"
set "BEST_NODE_MAJOR=!CAND_MAJOR!"
set "BEST_NODE_MINOR=!CAND_MINOR!"
set "BEST_NODE_PATCH=!CAND_PATCH!"
goto :eof
