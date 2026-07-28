#!/usr/bin/env bash
set -euo pipefail

sudo apt-get update
sudo apt-get install -y \
  libayatana-appindicator3-dev \
  libgtk-3-dev \
  librsvg2-dev \
  libwebkit2gtk-4.1-dev \
  libxdo-dev
