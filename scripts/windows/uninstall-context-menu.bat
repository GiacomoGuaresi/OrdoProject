@echo off
REM ==========================================
REM OrdoProject - Windows Context Menu Uninstaller
REM ==========================================

echo ==========================================
echo OrdoProject Context Menu Uninstaller
echo ==========================================
echo.
echo This will remove OrdoProject from the context menu.
echo.
pause

echo Removing context menu entries...

REM Remove registry keys
reg delete "HKEY_CURRENT_USER\Software\Classes\*\shell\OrdoSortFile" /f 2>nul
reg delete "HKEY_CURRENT_USER\Software\Classes\Directory\shell\OrdoSortFolder" /f 2>nul
reg delete "HKEY_CURRENT_USER\Software\Classes\Directory\Background\shell\OrdoSortAll" /f 2>nul

echo.
echo ==========================================
echo Context menu entries removed.
echo ==========================================
echo.
pause
