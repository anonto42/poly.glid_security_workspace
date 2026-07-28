#!/usr/bin/env bash
set -euo pipefail

moon run :format :check :test :build website:bundle automation:validate
