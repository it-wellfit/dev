@echo off
rem AuxBridge demo launcher (2026-11-17 canonical update: prod.db + dev-build-target)
set "DBPATH=%~dp0data\prod.db"
set "DBPATH=%DBPATH:\=/%"
set AUXBRIDGE_DB_URL=sqlite://%DBPATH%?mode=rw
set AUXBRIDGE_ADMIN_PASSWORD=Admin1234
set AUXBRIDGE_PORT=8090
set AUXBRIDGE_BIND=0.0.0.0
set "AUX_EXE=%LOCALAPPDATA%\Temp\dev-build-target\debug\auxbridge-server.exe"
"%AUX_EXE%" 1>"%LOCALAPPDATA%\Temp\kilo\auxbridge-demo.log" 2>&1