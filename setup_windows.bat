@echo off
setlocal EnableExtensions EnableDelayedExpansion
title MOSNA GUI — Installation Windows

rem =============================================================
rem  MOSNA GUI — Windows installer
rem  Double-click to run, or:  setup_windows.bat [--no-shortcut]
rem  Requirements: conda / miniconda (auto-installed if missing)
rem                and the Rust toolchain, from https://rustup.rs
rem
rem  Two halves are built here: the conda environment the Python
rem  analyses run in, and the Rust interface that drives them.
rem =============================================================

rem ── Configuration ────────────────────────────────────────────
set "ENV_NAME=mosna-GUI"
rem 3.11, not 3.10: the figures are drawn by `xy`, which requires it.
set "PYTHON_VERSION=3.11"

rem Resolve project root (folder containing this .bat)
set "PROJECT_DIR=%~dp0"
if "%PROJECT_DIR:~-1%"=="\" set "PROJECT_DIR=%PROJECT_DIR:~0,-1%"

set "GUI_BINARY=%PROJECT_DIR%\target\release\mosna-gui.exe"
set "MOSNA_PACKAGE=%PROJECT_DIR%\mosna-package"
set "RENDERER_DIR=%PROJECT_DIR%\python"
set "ICON_FILE=%PROJECT_DIR%\assets\logo.ico"
set "LAUNCHER_BAT=%PROJECT_DIR%\MosnaGUI.bat"
set "DESKTOP_SHORTCUT=%USERPROFILE%\Desktop\MOSNA GUI.lnk"

set "MINICONDA_DIR=%USERPROFILE%\miniconda3"
set "MINICONDA_INSTALLER=%TEMP%\Miniconda3-latest-Windows-x86_64.exe"
set "MINICONDA_URL=https://repo.anaconda.com/miniconda/Miniconda3-latest-Windows-x86_64.exe"

set "CONDA_BAT="
set "CREATE_SHORTCUT=1"

rem ── Parse optional arguments ─────────────────────────────────
for %%A in (%*) do (
    if /I "%%A"=="--no-shortcut" set "CREATE_SHORTCUT=0"
)

rem ── Banner ───────────────────────────────────────────────────
echo.
echo ============================================================
echo          MOSNA GUI ^— Windows Installation
echo ============================================================
echo.
echo  Project : %PROJECT_DIR%
echo.

rem ── Sanity checks ────────────────────────────────────────────
echo [1/7] Sanity checks...

if not exist "%PROJECT_DIR%\Cargo.toml" (
    echo [ERROR] The Rust interface was not found.
    echo         Run this script from the project root directory.
    goto :fail
)

if not exist "%MOSNA_PACKAGE%" (
    echo [ERROR] mosna-package directory not found:
    echo         %MOSNA_PACKAGE%
    goto :fail
)

if not exist "%RENDERER_DIR%" (
    echo [ERROR] The figure renderer was not found:
    echo         %RENDERER_DIR%
    goto :fail
)

rem Reported before conda spends ten minutes resolving an environment.
where cargo >nul 2>&1
if errorlevel 1 (
    echo [ERROR] cargo is not in your PATH.
    echo         The interface is a Rust program. Install the toolchain from
    echo         https://rustup.rs , open a new terminal, and run this again.
    goto :fail
)

cd /d "%PROJECT_DIR%"

rem ── Locate conda ────────────────────────────────────────────
call :find_conda
if errorlevel 1 (
    echo [INFO] Conda not found — downloading Miniconda automatically...
    call :install_miniconda
    if errorlevel 1 goto :fail
    call :find_conda
    if errorlevel 1 (
        echo [ERROR] Conda still not found after Miniconda installation.
        goto :fail
    )
)
echo [OK]   Conda: %CONDA_BAT%
echo.

rem ── Step 1: Environment ──────────────────────────────────────
echo [2/7] Conda environment "%ENV_NAME%"...
call :ensure_env
if errorlevel 1 goto :fail

rem ── Step 2: Conda packages ───────────────────────────────────
echo.
echo [3/7] Installing conda-forge packages...
call "%CONDA_BAT%" install -n "%ENV_NAME%" -y -c conda-forge ^
    "python=%PYTHON_VERSION%" ^
    pyyaml ^
    pandas ^
    pyarrow ^
    "scipy=1.13" ^
    scikit-learn ^
    networkx ^
    matplotlib ^
    seaborn ^
    scanpy ^
    tqdm ^
    lifelines ^
    ipykernel ^
    ipywidgets ^
    markdown
if errorlevel 1 (
    echo [ERROR] conda install failed.
    goto :fail
)
echo [OK]   Conda packages installed.

rem ── Step 3: mosna-package ────────────────────────────────────
echo.
echo [4/7] Installing mosna-package and the figure renderer...
call "%CONDA_BAT%" run -n "%ENV_NAME%" python -m pip install "%MOSNA_PACKAGE%" --quiet
if errorlevel 1 (
    echo [ERROR] mosna-package installation failed.
    goto :fail
)
call "%CONDA_BAT%" run -n "%ENV_NAME%" python -m pip install -e "%RENDERER_DIR%" --quiet
if errorlevel 1 (
    echo [ERROR] mosna_xy installation failed.
    goto :fail
)
echo [OK]   mosna-package and mosna_xy installed.

rem ── Step 4: Verify imports ───────────────────────────────────
echo.
echo [5/7] Verifying key imports...
call "%CONDA_BAT%" run -n "%ENV_NAME%" python -c ^
    "import yaml, pandas, mosna, tysserand, mosna_xy, xy; print('[OK]   All imports successful.')"
if errorlevel 1 (
    echo [ERROR] Import verification failed. Check the output above.
    goto :fail
)

rem ── Step 5: Build the interface ──────────────────────────────
echo.
echo [6/7] Building the interface ^(the first build takes a few minutes^)...
pushd "%PROJECT_DIR%"
cargo build --release --locked
set "BUILD_FAILED=%ERRORLEVEL%"
popd
if not "%BUILD_FAILED%"=="0" (
    echo [ERROR] cargo build failed. See the output above.
    goto :fail
)
if not exist "%GUI_BINARY%" (
    echo [ERROR] The build finished but %GUI_BINARY% is not there.
    goto :fail
)
echo [OK]   Interface built: %GUI_BINARY%

rem ── Step 5: Launcher + shortcut ──────────────────────────────
echo.
echo [7/7] Creating launcher and desktop shortcut...

rem Generate MosnaGUI.bat launcher.
rem
rem It activates the conda environment before starting the interface, which
rem is what puts the analyses' interpreter first on PATH: the interface runs
rem `python -m package.<module>` as a sub-process, so the interpreter left in
rem front is the one that runs them.
(
    echo @echo off
    echo rem Auto-generated by setup_windows.bat
    echo set "CONDA_BAT=%CONDA_BAT%"
    echo call "%%CONDA_BAT%%" activate "%ENV_NAME%"
    echo set "MOSNA_GUI_ROOT=%PROJECT_DIR%"
    echo cd /d "%PROJECT_DIR%"
    echo "%GUI_BINARY%" %%*
) > "%LAUNCHER_BAT%"
echo [OK]   Launcher: %LAUNCHER_BAT%

if "%CREATE_SHORTCUT%"=="1" (
    call :create_shortcut
    if errorlevel 1 (
        echo [WARN]  Could not create desktop shortcut ^(non-fatal^).
    ) else (
        echo [OK]   Shortcut: %DESKTOP_SHORTCUT%
    )
) else (
    echo [INFO]  Desktop shortcut skipped ^(--no-shortcut^).
)

rem ── Done ─────────────────────────────────────────────────────
echo.
echo ============================================================
echo  Installation complete!
echo ============================================================
echo.
echo  To launch: double-click  MosnaGUI.bat  in the project folder
echo             or the shortcut on your Desktop.
echo.
pause
goto :eof


rem =============================================================
rem  Subroutines
rem =============================================================

:find_conda
set "CONDA_BAT="
if exist "%MINICONDA_DIR%\condabin\conda.bat" (
    set "CONDA_BAT=%MINICONDA_DIR%\condabin\conda.bat"
    exit /b 0
)
if exist "%USERPROFILE%\anaconda3\condabin\conda.bat" (
    set "CONDA_BAT=%USERPROFILE%\anaconda3\condabin\conda.bat"
    exit /b 0
)
if exist "%USERPROFILE%\Miniconda3\condabin\conda.bat" (
    set "CONDA_BAT=%USERPROFILE%\Miniconda3\condabin\conda.bat"
    exit /b 0
)
rem Search PATH
for /f "delims=" %%I in ('where conda.bat 2^>nul') do (
    set "CONDA_BAT=%%I"
    exit /b 0
)
exit /b 1


:install_miniconda
echo.
echo ── Miniconda auto-install ──────────────────────────────────
echo.
echo [INFO] Downloading installer from:
echo        %MINICONDA_URL%
echo.
powershell -NoProfile -ExecutionPolicy Bypass -Command ^
    "try { Invoke-WebRequest -Uri '%MINICONDA_URL%' -OutFile '%MINICONDA_INSTALLER%' -UseBasicParsing } catch { Write-Error $_; exit 1 }"
if errorlevel 1 (
    echo [ERROR] Download failed. Check your internet connection.
    exit /b 1
)
if not exist "%MINICONDA_INSTALLER%" (
    echo [ERROR] Installer file missing after download.
    exit /b 1
)
echo [INFO] Installing Miniconda silently to:
echo        %MINICONDA_DIR%
start /wait "" "%MINICONDA_INSTALLER%" /InstallationType=JustMe /RegisterPython=0 /S /D=%MINICONDA_DIR%
if errorlevel 1 (
    echo [ERROR] Miniconda installer returned an error.
    exit /b 1
)
if not exist "%MINICONDA_DIR%\condabin\conda.bat" (
    echo [ERROR] Miniconda installed but conda.bat not found.
    exit /b 1
)
echo [OK]   Miniconda installed at %MINICONDA_DIR%
exit /b 0


:ensure_env
call "%CONDA_BAT%" env list > "%TEMP%\mosna_envs.txt" 2>nul
if errorlevel 1 (
    echo [ERROR] Cannot list conda environments.
    exit /b 1
)
findstr /C:"%ENV_NAME%" "%TEMP%\mosna_envs.txt" >nul 2>nul
if not errorlevel 1 (
    echo [OK]   Environment "%ENV_NAME%" already exists — reusing.
    del /f /q "%TEMP%\mosna_envs.txt" >nul 2>nul
    exit /b 0
)
echo [INFO] Creating environment "%ENV_NAME%" ...
call "%CONDA_BAT%" create -n "%ENV_NAME%" python=%PYTHON_VERSION% -y
if errorlevel 1 (
    echo [ERROR] conda env create failed.
    del /f /q "%TEMP%\mosna_envs.txt" >nul 2>nul
    exit /b 1
)
del /f /q "%TEMP%\mosna_envs.txt" >nul 2>nul
exit /b 0


:create_shortcut
set "ICON_ARG=%GUI_BINARY%"
if exist "%ICON_FILE%" set "ICON_ARG=%ICON_FILE%"

powershell -NoProfile -ExecutionPolicy Bypass -Command ^
    "$s = (New-Object -Com WScript.Shell).CreateShortcut('%DESKTOP_SHORTCUT%'); ^
    $s.TargetPath = '%LAUNCHER_BAT%'; ^
    $s.WorkingDirectory = '%PROJECT_DIR%'; ^
    $s.IconLocation = '%ICON_ARG%'; ^
    $s.Description = 'Launch MOSNA GUI'; ^
    $s.Save()"
exit /b %ERRORLEVEL%


:fail
echo.
echo ============================================================
echo  Installation FAILED — see errors above.
echo ============================================================
echo.
pause
exit /b 1
