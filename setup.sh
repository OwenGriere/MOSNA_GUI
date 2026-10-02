#!/usr/bin/env bash
# =============================================================
#  MOSNA GUI — Linux / macOS installer
#  Usage:  bash setup.sh [--no-shortcut] [--no-rust] [--dev]
#  Requirements: conda in PATH, and the Rust toolchain (cargo)
#
#  Two halves are built here. The analyses are Python and live in a
#  conda environment; the interface is Rust and is compiled with
#  cargo. Both are needed for the application to start, which is why
#  one script does both rather than leaving the second to be
#  discovered by the user when the launcher fails.
# =============================================================

set -euo pipefail

# ── Colour helpers ────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; BOLD='\033[1m'; RESET='\033[0m'
info()    { echo -e "${CYAN}[INFO]${RESET}  $*"; }
success() { echo -e "${GREEN}[OK]${RESET}    $*"; }
warn()    { echo -e "${YELLOW}[WARN]${RESET}  $*"; }
error()   { echo -e "${RED}[ERROR]${RESET} $*" >&2; }
step()    { echo -e "\n${BOLD}── $* ──${RESET}"; }

# ── Parse arguments ───────────────────────────────────────────
CREATE_SHORTCUT=true
BUILD_RUST=true
DEV_MODE=true
for arg in "$@"; do
    case "$arg" in
        --no-shortcut) CREATE_SHORTCUT=false ;;
        --no-rust)     BUILD_RUST=false ;;
        --dev)         DEV_MODE=true ;;
        --help|-h)
            echo "Usage: bash setup.sh [--no-shortcut] [--no-rust] [--dev]"
            echo "  --no-shortcut   Skip desktop launcher creation"
            echo "  --no-rust       Skip compiling the interface (environment only)"
            echo "  --dev           Install mosna-package in editable mode"
            exit 0 ;;
        *) warn "Unknown argument: $arg" ;;
    esac
done

# ── Project paths ─────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ENV_NAME="mosna-GUI"
# 3.11, not 3.10. The figures are drawn by `xy`, which declares
# Requires-Python >= 3.11, and the analyses and the renderer share one
# interpreter — so the environment the analyses live in is what has to be
# new enough.
PY_VER="3.11"
MOSNA_PACKAGE_DIR="${SCRIPT_DIR}/mosna-package"
RENDERER_DIR="${SCRIPT_DIR}/python"
GUI_BINARY="${SCRIPT_DIR}/target/release/mosna-gui"
LAUNCHER_SH="${SCRIPT_DIR}/MosnaGUI.sh"
APP_NAME="Mosna GUI"
ICON_FILE="${SCRIPT_DIR}/assets/logo.ico"

find_desktop() {
    # 1. Try XDG standard (covers most modern Linux DEs)
    if command -v xdg-user-dir &>/dev/null; then
        local xdg_desk
        xdg_desk="$(xdg-user-dir DESKTOP 2>/dev/null || true)"
        if [ -n "${xdg_desk}" ] && [ -d "${xdg_desk}" ]; then
            echo "${xdg_desk}"; return
        fi
    fi
    # 2. Try common names in order
    for candidate in \
        "${HOME}/Desktop" \
        "${HOME}/Bureau" \
        "${HOME}/Escritorio" \
        "${HOME}/Schreibtisch" \
        "${HOME}/Bureaublad" \
        "${HOME}/Рабочий стол" \
        "${HOME}/桌面" \
        "${HOME}/デスクトップ"; do
        if [ -d "${candidate}" ]; then
            echo "${candidate}"; return
        fi
    done
    # 3. Fallback: create ~/Desktop
    warn "No desktop directory found — using ${HOME}/Desktop"
    mkdir -p "${HOME}/Desktop"
    echo "${HOME}/Desktop"
}
DESKTOP_DIR="$(find_desktop)"

# ── Banner ────────────────────────────────────────────────────
echo -e "${BOLD}"
echo "╔══════════════════════════════════════════════╗"
echo "║         MOSNA GUI — Linux/macOS Setup        ║"
echo "╚══════════════════════════════════════════════╝"
echo -e "${RESET}"
info "Project directory : ${SCRIPT_DIR}"
info "Conda environment  : ${ENV_NAME}  (Python ${PY_VER})"
info "Interface          : Rust, built with cargo"
echo ""

# ── Step 0: Sanity checks ─────────────────────────────────────
step "Step 1/6 — Sanity checks"

if [ ! -f "${SCRIPT_DIR}/Cargo.toml" ] || [ ! -d "${SCRIPT_DIR}/crates/mosna-gui" ]; then
    error "The Rust interface was not found in ${SCRIPT_DIR}"
    error "Run this script from the project root directory."
    exit 1
fi

if [ ! -d "${MOSNA_PACKAGE_DIR}" ]; then
    error "mosna-package directory not found: ${MOSNA_PACKAGE_DIR}"
    exit 1
fi

if [ ! -d "${RENDERER_DIR}" ]; then
    error "The figure renderer was not found: ${RENDERER_DIR}"
    exit 1
fi

# Detect conda
if ! command -v conda &>/dev/null; then
    error "conda is not available in your PATH."
    echo ""
    echo "  Please install Miniconda first:"
    echo "  https://docs.conda.io/en/latest/miniconda.html"
    echo ""
    echo "  Then open a new terminal and re-run this script."
    exit 1
fi

# And cargo, unless the build was waived. Checked here rather than at the
# point of use so a missing toolchain is reported before conda spends ten
# minutes resolving an environment.
if ${BUILD_RUST} && ! command -v cargo &>/dev/null; then
    error "cargo is not available in your PATH."
    echo ""
    echo "  The interface is a Rust program. Install the toolchain with:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo ""
    echo "  Then open a new terminal and re-run this script."
    echo "  To build only the Python environment for now: bash setup.sh --no-rust"
    exit 1
fi

CONDA_BASE="$(conda info --base 2>/dev/null)"
if [ -z "${CONDA_BASE}" ]; then
    error "Cannot determine conda base directory."
    exit 1
fi

# shellcheck source=/dev/null
source "${CONDA_BASE}/etc/profile.d/conda.sh"
success "conda found at ${CONDA_BASE}"
${BUILD_RUST} && success "cargo found: $(cargo --version)"

# ── Step 1: Create / reuse environment ───────────────────────
step "Step 2/6 — Conda environment"

if conda env list | awk '{print $1}' | grep -qx "${ENV_NAME}"; then
    # An environment left over from a previous version is on Python 3.10, and
    # `xy` declares Requires-Python >= 3.11 — so reusing it silently would get
    # as far as installing the renderer and fail there, several minutes in,
    # with a message about a version nobody asked for. Checked here instead.
    EXISTING="$(conda run -n "${ENV_NAME}" python -c \
        'import sys; print(f"{sys.version_info[0]}.{sys.version_info[1]}")' 2>/dev/null || true)"
    if [ -n "${EXISTING}" ] && [ "$(printf '%s\n' "${PY_VER}" "${EXISTING}" | sort -V | head -1)" != "${PY_VER}" ]; then
        error "Environment '${ENV_NAME}' is on Python ${EXISTING}, and ${PY_VER} or newer is needed."
        error "The figures are drawn by xy, which requires Python ${PY_VER}."
        echo ""
        echo "  Remove it and run this script again:"
        echo "    conda env remove -n ${ENV_NAME}"
        echo ""
        exit 1
    fi
    warn "Environment '${ENV_NAME}' already exists (Python ${EXISTING:-unknown}) — skipping creation."
    warn "To rebuild from scratch:  conda env remove -n ${ENV_NAME}"
else
    info "Creating environment '${ENV_NAME}' with Python ${PY_VER} ..."
    conda create -y -n "${ENV_NAME}" \
        -c conda-forge \
        python="${PY_VER}" \
        scanpy \
        pip \
        "scipy==1.13" \
        pyyaml \
        ipykernel \
        ipywidgets \
        markdown \
        tqdm \
        lifelines
    success "Environment created."
fi

conda activate "${ENV_NAME}"

# ── Step 2: Install the Python halves ────────────────────────
step "Step 3/6 — mosna-package and the figure renderer"

cd "${MOSNA_PACKAGE_DIR}"
if ${DEV_MODE}; then
    info "Installing mosna-package in editable/dev mode ..."
    python -m pip install -e .
else
    info "Installing mosna-package ..."
    python -m pip install .
fi
cd "${SCRIPT_DIR}"
success "mosna-package installed."

# The renderer, which is what turns each analysis's figure specifications
# into a PNG and an interactive chart. Editable, so a change to a figure is
# picked up without reinstalling.
info "Installing the mosna_xy renderer ..."
python -m pip install -e "${RENDERER_DIR}"
success "mosna_xy installed."

# ── Step 3: Verify key imports ────────────────────────────────
step "Step 4/6 — Verifying imports"

python - <<'PYCHECK'
import importlib.util, sys
missing = []
for mod in ["yaml", "pandas", "mosna", "tysserand", "mosna_xy", "xy"]:
    if importlib.util.find_spec(mod) is None:
        missing.append(mod)
if missing:
    print(f"[ERROR] Missing modules after install: {', '.join(missing)}", file=sys.stderr)
    sys.exit(1)
print("[OK] All required modules are importable.")
PYCHECK
success "All Python dependencies satisfied."

# ── Step 4: Build the interface ───────────────────────────────
step "Step 5/6 — Building the interface"

if ${BUILD_RUST}; then
    info "Compiling (the first build takes a few minutes) ..."
    # From the project root, so the workspace is the one in this directory
    # whatever the caller's working directory was.
    ( cd "${SCRIPT_DIR}" && cargo build --release --locked )
    if [ ! -x "${GUI_BINARY}" ]; then
        error "The build finished but ${GUI_BINARY} is not there."
        exit 1
    fi
    success "Interface built: ${GUI_BINARY}"
else
    info "Skipped (--no-rust)."
    if [ ! -x "${GUI_BINARY}" ]; then
        warn "No interface binary at ${GUI_BINARY} — the launcher will not start."
    fi
fi

# ── Step 5: Desktop launcher ──────────────────────────────────
step "Step 6/6 — Desktop launcher"

# Always regenerate MosnaGUI.sh with current resolved paths.
#
# The launcher activates the conda environment *before* starting the
# interface, which is what puts the analyses' interpreter first on PATH.
# The interface starts `python -m package.<module>` as a sub-process, so it
# is the interpreter the launcher leaves in front that runs them.
cat > "${LAUNCHER_SH}" <<LAUNCHEOF
#!/usr/bin/env bash
# Auto-generated by setup.sh — do not edit manually
set -e
source "${CONDA_BASE}/etc/profile.d/conda.sh"
conda activate "${ENV_NAME}"
# Named explicitly rather than left to be discovered: an interface started
# from a copied binary, or from a file manager, has no way of knowing which
# checkout it belongs to.
export MOSNA_GUI_ROOT="${SCRIPT_DIR}"
export MOSNA_PYTHON="\$(command -v python)"
cd "${SCRIPT_DIR}"
exec "${GUI_BINARY}" "\$@"
LAUNCHEOF
chmod +x "${LAUNCHER_SH}"
success "Launcher created: ${LAUNCHER_SH}"

if ${CREATE_SHORTCUT}; then
    OS="$(uname -s)"

    if [ "${OS}" = "Linux" ]; then
        DESKTOP_FILE="${DESKTOP_DIR}/MosnaGUI.desktop"
        mkdir -p "${DESKTOP_DIR}"

        ICON_LINE=""
        [ -f "${ICON_FILE}" ] && ICON_LINE="Icon=${ICON_FILE}"

        cat > "${DESKTOP_FILE}" <<DESKEOF
[Desktop Entry]
Type=Application
Name=${APP_NAME}
Comment=Spatial network analysis GUI (MOSNA + Tysserand)
Exec=/bin/bash -lc "${LAUNCHER_SH}"
${ICON_LINE}
Terminal=false
Categories=Science;Biology;Utility;
StartupNotify=true
StartupWMClass=mosna-gui
DESKEOF
        chmod +x "${DESKTOP_FILE}"

        # Mark trusted on GNOME (suppresses the "untrusted" warning)
        if command -v gio &>/dev/null; then
            gio set "${DESKTOP_FILE}" metadata::trusted true 2>/dev/null || true
        fi

        success "Desktop shortcut: ${DESKTOP_FILE}"
        warn "On GNOME: right-click the icon → 'Allow Launching' if still needed."

    elif [ "${OS}" = "Darwin" ]; then
        DESKTOP_FILE="${DESKTOP_DIR}/MosnaGUI.command"
        cp "${LAUNCHER_SH}" "${DESKTOP_FILE}"
        chmod +x "${DESKTOP_FILE}"
        success "macOS launcher: ${DESKTOP_FILE}"
        warn "Double-click MosnaGUI.command to start."
        warn "If blocked by Gatekeeper: System Settings → Privacy & Security → Allow."

    else
        warn "Unknown OS '${OS}' — desktop shortcut skipped."
        warn "Launch manually with:  bash ${LAUNCHER_SH}"
    fi
else
    info "Desktop shortcut skipped (--no-shortcut)."
fi

# Once installed, the Windows installer and uninstaller are of no use in this
# folder. INSTALLATION.exe does the same the other way round.
for name in INSTALLATION.exe UNINSTALL.exe; do
    if [ -f "${SCRIPT_DIR}/${name}" ]; then
        rm -f "${SCRIPT_DIR}/${name}"
        info "removed ${name} (Windows only)"
    fi
done

# ── Done ──────────────────────────────────────────────────────
echo ""
echo -e "${GREEN}${BOLD}"
echo "╔══════════════════════════════════════════════╗"
echo "║        Installation complete  ✓              ║"
echo "╚══════════════════════════════════════════════╝"
echo -e "${RESET}"
echo "  To launch manually:"
echo -e "    ${CYAN}bash ${LAUNCHER_SH}${RESET}"
echo ""
