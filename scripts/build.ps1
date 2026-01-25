# qsl-cardhub 构建脚本 (Windows PowerShell)
# 用途：自动化构建 Tauri 应用，包括前端构建和应用打包

$ErrorActionPreference = "Stop"  # 任何命令失败立即退出

# 打印带颜色的消息
function Print-Success {
    param([string]$Message)
    Write-Host "✓ $Message" -ForegroundColor Green
}

function Print-Error {
    param([string]$Message)
    Write-Host "✗ $Message" -ForegroundColor Red
}

function Print-Warning {
    param([string]$Message)
    Write-Host "⚠ $Message" -ForegroundColor Yellow
}

function Print-Info {
    param([string]$Message)
    Write-Host "ℹ $Message" -ForegroundColor Cyan
}

function Print-Step {
    param([string]$Message)
    Write-Host ""
    Write-Host "==> $Message" -ForegroundColor Blue
}

# 获取版本号
function Get-AppVersion {
    $cargoContent = Get-Content "Cargo.toml"
    $versionLine = $cargoContent | Where-Object { $_ -match '^version = "(.+)"' }
    if ($versionLine -match '"(.+)"') {
        return $Matches[1]
    }
    return "0.0.0"
}

# 检查命令是否存在
function Test-CommandExists {
    param([string]$Command)
    $null = Get-Command $Command -ErrorAction SilentlyContinue
    return $?
}

# 检查依赖
function Test-Dependencies {
    Print-Step "步骤 1/5: 检查依赖"

    $missingDeps = @()

    # 检查 Node.js
    if (Test-CommandExists "node") {
        $nodeVersion = node --version
        Print-Success "Node.js: $nodeVersion"
    } else {
        $missingDeps += "Node.js"
        Print-Error "Node.js 未安装"
    }

    # 检查 pnpm
    if (Test-CommandExists "pnpm") {
        $pnpmVersion = pnpm --version
        Print-Success "pnpm: v$pnpmVersion"
    } else {
        $missingDeps += "pnpm"
        Print-Error "pnpm 未安装"
    }

    # 检查 Rust
    if (Test-CommandExists "rustc") {
        $rustVersion = (rustc --version).Split()[1]
        Print-Success "Rust: v$rustVersion"
    } else {
        $missingDeps += "Rust"
        Print-Error "Rust 未安装"
    }

    # 检查 cargo
    if (Test-CommandExists "cargo") {
        $cargoVersion = (cargo --version).Split()[1]
        Print-Success "cargo: v$cargoVersion"
    } else {
        $missingDeps += "cargo"
        Print-Error "cargo 未安装"
    }

    # 如果有缺失的依赖，提示并退出
    if ($missingDeps.Count -gt 0) {
        Write-Host ""
        Print-Error "缺少以下依赖：$($missingDeps -join ', ')"
        Write-Host ""
        Write-Host "请安装缺失的依赖："
        foreach ($dep in $missingDeps) {
            switch ($dep) {
                "Node.js" { Write-Host "  - Node.js: https://nodejs.org/" }
                "pnpm" { Write-Host "  - pnpm: https://pnpm.io/" }
                "Rust" { Write-Host "  - Rust: https://rustup.rs/" }
                "cargo" { Write-Host "  - cargo (随 Rust 一起安装)" }
            }
        }
        exit 1
    }

    Write-Host ""
}

# 检查版本号一致性
function Test-VersionConsistency {
    $cargoVersion = Get-AppVersion
    $tauriContent = Get-Content "tauri.conf.json" | ConvertFrom-Json
    $tauriVersion = $tauriContent.version

    if ($cargoVersion -ne $tauriVersion) {
        Print-Warning "版本号不一致："
        Write-Host "  Cargo.toml: $cargoVersion"
        Write-Host "  tauri.conf.json: $tauriVersion"
        Write-Host ""
        Print-Info "建议运行: .\scripts\sync-version.ps1"
        Write-Host ""
    }
}

# 构建前端
function Build-Frontend {
    Print-Step "步骤 2/5: 构建前端"

    Push-Location web

    try {
        # 检查是否需要安装依赖
        if (-not (Test-Path "node_modules")) {
            Print-Info "安装前端依赖..."
            pnpm install
        }

        Print-Info "构建前端..."
        pnpm run build

        if (Test-Path "dist") {
            Print-Success "前端构建完成"
        } else {
            Print-Error "前端构建失败"
            exit 1
        }
    } finally {
        Pop-Location
    }

    Write-Host ""
}

# 打包应用
function Build-App {
    Print-Step "步骤 3/5: 打包 Tauri 应用"

    Print-Info "开始 Tauri 打包..."
    cargo tauri build

    Print-Success "Tauri 打包完成"
    Write-Host ""
}

# 整理产物
function Copy-BuildArtifact {
    Print-Step "步骤 4/5: 整理构建产物"

    $version = Get-AppVersion
    $arch = if ([System.Environment]::Is64BitOperatingSystem) {
        if ([System.Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture -eq "Arm64") {
            "arm64"
        } else {
            "x64"
        }
    } else {
        "x86"
    }
    $outputName = "qsl-cardhub-v$version-windows-$arch.msi"

    # 创建 dist 目录
    New-Item -ItemType Directory -Force -Path "dist" | Out-Null

    # 查找 MSI 文件
    $msiPath = Get-ChildItem -Path "target\release\bundle\msi" -Filter "*.msi" -Recurse -ErrorAction SilentlyContinue | Select-Object -First 1

    if ($null -eq $msiPath) {
        Print-Error "未找到 MSI 文件"
        exit 1
    }

    # 复制到 dist 目录
    Copy-Item $msiPath.FullName "dist\$outputName"

    Print-Success "产物已复制到: dist\$outputName"
    Write-Host ""
}

# 验证产物
function Test-BuildArtifact {
    Print-Step "步骤 5/5: 验证构建产物"

    $version = Get-AppVersion
    $arch = if ([System.Environment]::Is64BitOperatingSystem) {
        if ([System.Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture -eq "Arm64") {
            "arm64"
        } else {
            "x64"
        }
    } else {
        "x86"
    }
    $outputFile = "dist\qsl-cardhub-v$version-windows-$arch.msi"

    # 检查文件存在
    if (-not (Test-Path $outputFile)) {
        Print-Error "构建产物不存在: $outputFile"
        exit 1
    }

    # 检查文件大小
    $fileInfo = Get-Item $outputFile
    $fileSizeMB = [math]::Round($fileInfo.Length / 1MB, 2)

    if ($fileInfo.Length -lt 5MB) {
        Print-Warning "文件大小过小 (${fileSizeMB}MB)，可能构建不完整"
    } elseif ($fileInfo.Length -gt 100MB) {
        Print-Warning "文件大小过大 (${fileSizeMB}MB)，可能包含不必要的文件"
    } else {
        Print-Success "文件大小正常: ${fileSizeMB}MB"
    }

    Print-Success "构建产物验证通过"
    Write-Host ""
}

# 打印构建总结
function Write-BuildSummary {
    param([int]$BuildTime)

    $version = Get-AppVersion
    $arch = if ([System.Environment]::Is64BitOperatingSystem) {
        if ([System.Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture -eq "Arm64") {
            "arm64"
        } else {
            "x64"
        }
    } else {
        "x86"
    }
    $outputFile = "dist\qsl-cardhub-v$version-windows-$arch.msi"
    $fileInfo = Get-Item $outputFile
    $fileSizeMB = [math]::Round($fileInfo.Length / 1MB, 2)

    Write-Host ""
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    Print-Success "构建完成！"
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    Write-Host ""
    Write-Host "版本号:   $version"
    Write-Host "产物路径: $outputFile"
    Write-Host "文件大小: ${fileSizeMB}MB"
    Write-Host "构建用时: $BuildTime 秒"
    Write-Host ""
}

# 主函数
function Main {
    $startTime = Get-Date

    Write-Host ""
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    Write-Host "  qsl-cardhub 构建脚本 (Windows)"
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    Test-Dependencies
    Test-VersionConsistency
    Build-Frontend
    Build-App
    Copy-BuildArtifact
    Test-BuildArtifact

    $endTime = Get-Date
    $buildTime = [math]::Round(($endTime - $startTime).TotalSeconds)
    Write-BuildSummary -BuildTime $buildTime
}

# 启动构建
try {
    Main
} catch {
    Print-Error "构建失败: $_"
    exit 1
}
