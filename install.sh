#!/usr/bin/env bash

set -e -o pipefail

readonly MAGENTA="$(tput setaf 5 2>/dev/null || echo '')"
readonly GREEN="$(tput setaf 2 2>/dev/null || echo '')"
readonly CYAN="$(tput setaf 6 2>/dev/null || echo '')"
readonly ORANGE="$(tput setaf 3 2>/dev/null || echo '')"
readonly NO_COLOR="$(tput sgr0 2>/dev/null || echo '')"

if ! command -v curl >/dev/null 2>&1; then
    echo "Error: curl is required to install oh-my-droid."
    exit 1
fi

if ! command -v tar >/dev/null 2>&1; then
    echo "Error: tar is required to install oh-my-droid."
    exit 1
fi

export PATH="$HOME/.local/bin:$PATH"

RELEASE_URL="https://api.github.com/repos/tsirysndr/oh-my-droid/releases/latest"

function detect_os() {
  # Determine the operating system
  OS=$(uname -s)
  if [ "$OS" = "Linux" ]; then
    # Determine the CPU architecture
    ARCH=$(uname -m)
    if [ "$ARCH" = "aarch64" ]; then
      ASSET_NAME="_aarch64-unknown-linux-gnu.tar.gz"
    elif [ "$ARCH" = "x86_64" ]; then
        ASSET_NAME="_x86_64-unknown-linux-gnu.tar.gz"
    else
        echo "Unsupported architecture: $ARCH"
        exit 1
    fi
  else
      echo "Unsupported operating system: $OS"
      echo "This script only supports Linux."
      exit 1
  fi;
}

detect_os

# Retrieve the download URL for the desired asset
DOWNLOAD_URL=$(curl -sSL $RELEASE_URL | grep -o "browser_download_url.*$ASSET_NAME\"" | cut -d ' ' -f 2)

ASSET_NAME=$(basename $DOWNLOAD_URL)

INSTALL_DIR="/usr/local/bin"

DOWNLOAD_URL=`echo $DOWNLOAD_URL | tr -d '\"'`

# Download the asset
curl -SL $DOWNLOAD_URL -o /tmp/$ASSET_NAME

# Extract the asset
tar -xzf /tmp/$ASSET_NAME -C /tmp

# Set the correct permissions for the binary
chmod +x /tmp/oh-my-droid

if command -v sudo >/dev/null 2>&1; then
    sudo mv /tmp/oh-my-droid $INSTALL_DIR
else
    mv /tmp/oh-my-droid $INSTALL_DIR
fi

# Clean up temporary files
rm /tmp/$ASSET_NAME

cat << EOF
${CYAN}
        ______                              _________            ______________
  _________  /_     _______ ________  __    ______  /_______________(_)_____  /
  _  __ \\_  __ \\    __  __ `__ \\_  / / /    _  __  /__  ___/  __ \\_  /_  __  /
  / /_/ /  / / /    _  / / / / /  /_/ /     / /_/ / _  /   / /_/ /  / / /_/ /
  \\____//_/ /_/     /_/ /_/ /_/_\\__, /      \\__,_/  /_/    \\____//_/  \\__,_/
                              /____/

${NO_COLOR}

Opinionated Android 15+ Linux Terminal Setup

${GREEN}https://github.com/tsirysndr/oh-my-droid${NO_COLOR}

Please file an issue if you encounter any problems!

===============================================================================

Installation completed! 🎉

EOF

oh-my-droid setup

