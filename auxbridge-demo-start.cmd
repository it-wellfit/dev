@echo off
rem AuxBridge demo launcher (2026-11-11 naming decree; ASCII-only + CRLF: cmd batch must not be UTF-8 with Chinese paths)
set AUXBRIDGE_DB_URL=sqlite://../data/demo-auxbridge.db?mode=rwc
set AUXBRIDGE_ADMIN_PASSWORD=Admin1234
set AUXBRIDGE_PORT=8090
set AUXBRIDGE_BIND=0.0.0.0
cd /d %~dp0platform
set "AUX_EXE=%LOCALAPPDATA%\Temp\dev-target\debug\auxbridge-server.exe"
"%AUX_EXE%" 1>"%LOCALAPPDATA%\Temp\kilo\auxbridge-demo.log" 2>&1
