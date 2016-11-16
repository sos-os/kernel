#!/bin/bash
#
#  SOS: the Stupid Operating System
#  by Eliza Weisman (hi@hawkweisman.me)
#
#  Copyright (c) 2015 Eliza Weisman
#  Released under the terms of the MIT license. See `LICENSE` in the root
#  directory of this repository for more information.
#
#  Portions of this script are adapted from a script by Steve Klabnik for
#  the IntermezzOS project, available at
#  http://intermezzos.github.io/book/appendix/osx-install.html
#
set -e
# this script will install the required dependencies and tools
# to build the SOS kernel.
bold=$(tput bold)
normal=$(tput sgr0)
# check if `brew` is installed
command -v brew >/dev/null 2>&1
if [ $? -eq 1 ]
then
    echo "${bold}install-mac:${normal} Homebrew is not installed."
    echo "${bold}install-mac:${normal} Please go to http://brew.sh/ to install it before continuing."
    exit
fi

export PREFIX="$HOME/opt/"
export TARGET=x86_64-pc-elf
export PATH="$PREFIX/bin:$PATH"

mkdir -p $HOME/src
mkdir -p $PREFIX

# dependencies installable with brew
echo "${bold}install-mac:${normal} Installing dependencies using Homebrew..."
brew update | sed "s/^/${bold}brew:${normal} /"
brew tap Homebrew/bundle | sed "s/^/${bold}brew:${normal} /"
brew bundle | sed "s/^/${bold}brew:${normal} /"


CARGO_CONFIG="$HOME/.cargo/config"
GREP_TARGET_LINKER="\[target\.x86_64\-sos\-kernel\-gnu\]"
TARGET_LINKER="\n\n[target.x86_64-sos-kernel-gnu]\nlinker = \"/usr/local/bin/x86_64-pc-elf-gcc\""

if grep -q $GREP_TARGET_LINKER "$CARGO_CONFIG"; then
    echo "${bold}install-mac:${normal} Target linker already present in $CARGO_CONFIG. Done."
else
    echo "${bold}install-mac:${normal} Adding target linker to $CARGO_CONFIG..."
    printf "$TARGET_LINKER" >> "$CARGO_CONFIG"
fi
