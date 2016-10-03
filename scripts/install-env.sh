#!/bin/bash
#
#  SOS: the Stupid Operating System
#  by Eliza Weisman (hi@hawkweisman.me)
#
#  Copyright (c) 2015 Eliza Weisman
#  Released under the terms of the MIT license. See `LICENSE` in the root
#  directory of this repository for more information.
#
set -e
# this script will install the required dependencies and tools
# to build the SOS kernel.

CONTINUE=false
platform=$(uname)
bold=$(tput bold)
normal=$(tput sgr0)

echo ""
echo "${bold}install:${normal} Stupid Operating System Dev Environment Setup *** "
echo ""
echo "${bold}install:${normal} This script is about to download and install software on your computer."
if [[ $PLATFORM == 'Darwin' ]]; then
    echo "${bold}install:${normal} Since you are on macOS, this install process will not require sudo"
else
    echo "${bold}install:${normal} Depending on your OS and package manager, this process may require sudo."
fi
echo "${bold}install:${normal} Please take the time to read the script source code and ensure you are"
echo "${bold}install:${normal} aware of what software will be installed before continuing."
echo ""
echo "${bold}install:${normal} Do you want to continue? (y/n)"

read -r response
if [[ $response =~ ^([yY][eE][sS]|[yY])$ ]]; then
    CONTINUE=true
fi

if ! $CONTINUE; then
    echo "${bold}install:${normal} Okay, cancelling installation."
    exit
fi

echo "${bold}install:${normal} Checking if Rust is installed..."
command -v rustc >/dev/null 2>&1
if [[ $? -eq 0 ]]; then
    command -v rustup >/dev/null 2>&1
    if [[ $? -eq 1 ]]; then
        echo "${bold}install:${normal} Rust is installed, but it is not managed by \`rustup\`."
        echo "${bold}install:${normal} Your current Rust installation is not supported."
        echo "${bold}install:${normal} Please visit https://www.rustup.rs to re-install using \`rustup\`."
        echo "${bold}install:${normal} Exiting."
        exit 1
    else
        echo "${bold}install:${normal} Rust is already installed."
    fi
else
    echo ""
    echo "${bold}install:${normal} installing Rust"
    echo ""
    curl https://sh.rustup.rs -sSf | sh
fi


echo "${bold}install:${normal} Updating Rust version"
rustup update nightly
echo "${bold}install:${normal} Overriding default Rust to nightly for SOS"
rustup override add nightly

echo "${bold}install:${normal} Installing platform-specific dependencies."
case $platform in
    Darwin)
        echo "${bold}install:${normal} Detected OS as macOS."
        ./scripts/install-env-mac.sh
        ;;
    Linux)
        echo "${bold}install:${normal} Detected OS as Linux."
        ./scripts/install-env-linux.sh
        ;;
    *)
        echo "${bold}install:${normal} Unknown OS, exiting."
        exit 1
        ;;
esac

echo ""
echo "${bold}install:${normal} Installing \`xargo\`."
echo ""
cargo install xargo
