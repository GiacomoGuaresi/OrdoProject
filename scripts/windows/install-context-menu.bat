@echo off
REM ==========================================
REM OrdoProject - Windows Context Menu Installer
REM ==========================================
REM This script automatically detects the OrdoProject executable
REM and creates a temporary registry file with the correct paths

setlocal enabledelayedexpansion

echo ==========================================
echo OrdoProject Context Menu Installer
echo ==========================================
echo.

REM Try to find the executable in common locations
set "ORDO_EXE="

REM Check in dist/win-unpacked (development build)
if exist "%~dp0..\..\dist\win-unpacked\ordoproject.exe" (
    set "ORDO_EXE=%~dp0..\..\dist\win-unpacked\ordoproject.exe"
    echo Found executable in dist/win-unpacked
)

REM Check in parent directory (portable installation)
if not defined ORDO_EXE (
    if exist "%~dp0..\..\ordoproject.exe" (
        set "ORDO_EXE=%~dp0..\..\ordoproject.exe"
        echo Found executable in root directory
    )
)

REM Check in Program Files
if not defined ORDO_EXE (
    if exist "%ProgramFiles%\OrdoProject\ordoproject.exe" (
        set "ORDO_EXE=%ProgramFiles%\OrdoProject\ordoproject.exe"
        echo Found executable in Program Files
    )
)

REM Check in Local AppData
if not defined ORDO_EXE (
    if exist "%LocalAppData%\Programs\OrdoProject\ordoproject.exe" (
        set "ORDO_EXE=%LocalAppData%\Programs\OrdoProject\ordoproject.exe"
        echo Found executable in Local AppData
    )
)

REM If not found, ask user
if not defined ORDO_EXE (
    echo.
    echo ERROR: Could not find ordoproject.exe automatically.
    echo Please enter the full path to ordoproject.exe:
    set /p "ORDO_EXE="
)

REM Verify the file exists
if not exist "!ORDO_EXE!" (
    echo.
    echo ERROR: File not found: !ORDO_EXE!
    echo Please check the path and try again.
    pause
    exit /b 1
)

REM Convert path to registry format (escape backslashes)
set "ORDO_EXE_REG=!ORDO_EXE:\=\\!"

echo.
echo Using executable: !ORDO_EXE!
echo.

REM Create temporary registry file
set "TEMP_REG=%TEMP%\OrdoProject_ContextMenu.reg"

echo Creating registry file...
(
echo Windows Registry Editor Version 5.00
echo.
echo ; ==========================================
echo ; File singoli o multipli
echo ; ==========================================
echo [HKEY_CURRENT_USER\Software\Classes\*\shell\OrdoSortFile]
echo @="Sort this with Ordo"
echo "Icon"="!ORDO_EXE_REG!"
echo.
echo [HKEY_CURRENT_USER\Software\Classes\*\shell\OrdoSortFile\command]
echo @="\"!ORDO_EXE_REG!\" \"%%1\""
echo.
echo ; ==========================================
echo ; Cartella cliccata ^(non sfondo^)
echo ; ==========================================
echo [HKEY_CURRENT_USER\Software\Classes\Directory\shell\OrdoSortFolder]
echo @="Sort this folder with Ordo"
echo "Icon"="!ORDO_EXE_REG!"
echo.
echo [HKEY_CURRENT_USER\Software\Classes\Directory\shell\OrdoSortFolder\command]
echo @="\"!ORDO_EXE_REG!\" \"%%1\""
echo.
echo ; ==========================================
echo ; Sfondo cartella aperta ^(tutti i file nella cartella^)
echo ; ==========================================
echo [HKEY_CURRENT_USER\Software\Classes\Directory\Background\shell\OrdoSortAll]
echo @="Sort all with Ordo"
echo "Icon"="!ORDO_EXE_REG!"
echo.
echo [HKEY_CURRENT_USER\Software\Classes\Directory\Background\shell\OrdoSortAll\command]
echo @="\"!ORDO_EXE_REG!\" \"%%V\" ALL"
) > "!TEMP_REG!"

echo Registry file created: !TEMP_REG!
echo.
echo Installing context menu entries...

REM Import the registry file
regedit /s "!TEMP_REG!"

if %ERRORLEVEL% EQU 0 (
    echo.
    echo ==========================================
    echo SUCCESS! Context menu entries installed.
    echo ==========================================
    echo.
    echo You can now right-click on:
    echo   - Files: "Sort this with Ordo"
    echo   - Folders: "Sort this folder with Ordo"
    echo   - Folder background: "Sort all with Ordo"
    echo.
) else (
    echo.
    echo ERROR: Failed to install context menu entries.
    echo Please run this script as Administrator.
    echo.
)

REM Clean up temporary file
del "!TEMP_REG!" 2>nul

pause
