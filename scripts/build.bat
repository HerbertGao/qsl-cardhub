@echo off
REM qsl-cardhub 构建脚本入口 (Windows CMD)
REM 调用 PowerShell 脚本执行实际构建

echo.
echo ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
echo   qsl-cardhub 构建脚本
echo ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
echo.

REM 检查 PowerShell 是否可用
where powershell >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo [错误] PowerShell 未找到
    echo.
    echo 请确保 PowerShell 已安装。Windows 10+ 默认包含 PowerShell。
    exit /b 1
)

REM 执行 PowerShell 脚本
powershell -ExecutionPolicy Bypass -File "%~dp0build.ps1"

REM 传递退出码
exit /b %ERRORLEVEL%
