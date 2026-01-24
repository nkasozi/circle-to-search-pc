#!/usr/bin/env pwsh

$ErrorActionPreference = "Stop"
$APP_NAME = "Circle to Search"
$EXECUTABLE_NAME = "circle-to-search-pc.exe"
$INSTALL_DIR = "$env:LOCALAPPDATA\$APP_NAME"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Building $APP_NAME for Windows" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

Write-Host "[1/6] Checking prerequisites..." -ForegroundColor Yellow
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "ERROR: Rust/Cargo not found. Please install from https://rustup.rs" -ForegroundColor Red
    exit 1
}
Write-Host "  Cargo found" -ForegroundColor Green

if (-not (Test-Path "tessdata/eng.traineddata")) {
    Write-Host "  tessdata not found, downloading..." -ForegroundColor Yellow
    try {
        if (Get-Command bash -ErrorAction SilentlyContinue) {
            bash ./download-tessdata.sh
        } else {
            New-Item -ItemType Directory -Path "tessdata" -Force | Out-Null
            Write-Host "  Downloading English language data..." -ForegroundColor Yellow
            Invoke-WebRequest -Uri "https://github.com/tesseract-ocr/tessdata_fast/raw/main/eng.traineddata" -OutFile "tessdata/eng.traineddata"
        }
        if (-not (Test-Path "tessdata/eng.traineddata")) {
            throw "Download failed"
        }
    } catch {
        Write-Host "ERROR: Failed to download tessdata" -ForegroundColor Red
        exit 1
    }
}
Write-Host "  tessdata found" -ForegroundColor Green

Write-Host ""
Write-Host "[2/6] Building release binary..." -ForegroundColor Yellow
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Build failed" -ForegroundColor Red
    exit 1
}
Write-Host "  Build successful" -ForegroundColor Green

Write-Host ""
Write-Host "[3/6] Stopping any running instance..." -ForegroundColor Yellow
$running_process = Get-Process -Name "circle-to-search-pc" -ErrorAction SilentlyContinue
if ($running_process) {
    Stop-Process -Name "circle-to-search-pc" -Force
    Start-Sleep -Seconds 1
    Write-Host "  Stopped existing instance" -ForegroundColor Green
} else {
    Write-Host "  No running instance found" -ForegroundColor Green
}

Write-Host ""
Write-Host "[4/6] Creating installation directory..." -ForegroundColor Yellow
if (Test-Path $INSTALL_DIR) {
    Remove-Item -Path $INSTALL_DIR -Recurse -Force
    Write-Host "  Removed old installation" -ForegroundColor Green
}
New-Item -ItemType Directory -Path $INSTALL_DIR -Force | Out-Null
New-Item -ItemType Directory -Path "$INSTALL_DIR\tessdata" -Force | Out-Null
Write-Host "  Created: $INSTALL_DIR" -ForegroundColor Green

Write-Host ""
Write-Host "[5/6] Copying files..." -ForegroundColor Yellow
Copy-Item "target\release\$EXECUTABLE_NAME" -Destination "$INSTALL_DIR\" -Force
Write-Host "  Copied executable" -ForegroundColor Green

Copy-Item "tessdata\*" -Destination "$INSTALL_DIR\tessdata\" -Recurse -Force
Write-Host "  Copied tessdata" -ForegroundColor Green

if (Test-Path "assets\icons\icon.png") {
    Copy-Item "assets\icons\icon.png" -Destination "$INSTALL_DIR\" -Force
    Write-Host "  Copied icon" -ForegroundColor Green
}

Write-Host ""
Write-Host "[6/6] Creating Start Menu shortcut..." -ForegroundColor Yellow
$WshShell = New-Object -ComObject WScript.Shell
$START_MENU_DIR = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs"
$SHORTCUT_PATH = "$START_MENU_DIR\$APP_NAME.lnk"
$Shortcut = $WshShell.CreateShortcut($SHORTCUT_PATH)
$Shortcut.TargetPath = "$INSTALL_DIR\$EXECUTABLE_NAME"
$Shortcut.WorkingDirectory = $INSTALL_DIR
$Shortcut.Description = "Circle to Search - Screen capture with OCR and Google Lens search"
$Shortcut.Save()
Write-Host "  Created Start Menu shortcut" -ForegroundColor Green

Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "  Installation Complete!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""
Write-Host "Installed to: $INSTALL_DIR" -ForegroundColor Cyan
Write-Host ""
Write-Host "To launch, either:" -ForegroundColor White
Write-Host "  1. Search '$APP_NAME' in Start Menu" -ForegroundColor White
Write-Host "  2. Run: $INSTALL_DIR\$EXECUTABLE_NAME" -ForegroundColor White
Write-Host ""
Write-Host "To uninstall, simply delete:" -ForegroundColor Yellow
Write-Host "  - $INSTALL_DIR" -ForegroundColor Yellow
Write-Host "  - $SHORTCUT_PATH" -ForegroundColor Yellow
Write-Host ""
