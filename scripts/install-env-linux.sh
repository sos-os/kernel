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

CONTINUE=false

distro=$(lsb_release -is)
platform=$(uname)
bold=$(tput bold)
normal=$(tput sgr0)

echo "${bold}install-env-linux:${normal} Distro is ${distro}"
case $distro in
    Ubuntu | Debian)
        echo "${bold}install-env-linux:${normal} Installing with apt-get."
        echo "${bold}install-env-linux:${normal} This will require sudo."
        echo "${bold}install-env-linux:${normal} Do you want to continue? (y/n)"

        read -r response
        if [[ $response =~ ^([yY][eE][sS]|[yY])$ ]]; then
            CONTINUE=true
        fi

        if ! $CONTINUE; then
            echo "${bold}install-env-linux:${normal} Okay, cancelling installation."
            exit
        fi

        sudo apt-get install nasm xorriso qemu build-essential | sed "s/^/${bold}apt-get:${normal} /"
        ;;
    Arch | ManjaroLinux)
        echo "${bold}install-env-linux:${normal} Installing with pacman."
        echo "${bold}install-env-linux:${normal} This will require sudo."
        echo "${bold}install-env-linux:${normal} Do you want to continue? (y/n)"

        read -r response
        if [[ $response =~ ^([yY][eE][sS]|[yY])$ ]]; then
            CONTINUE=true
        fi

        if ! $CONTINUE; then
            echo "${bold}install-env-linux:${normal} Okay, cancelling installation."
            exit
        fi

        sudo pacman -S --needed binutils grub libisoburn nasm qemu | sed "s/^/${bold}pacman:${normal} /"
        ;;
esac
    # todo: support non-x86_64 architectures here (later)

    gcc = $(which gcc)
    echo "${bold}install-env-linux:${normal} Linking ${gcc} to /usr/bin/x86_64-pc-elf-gcc."
    sudo ln -s $gcc /usr/bin/x86_64-pc-elf-gcc

    objcopy = $(which objcopy)
    echo "${bold}install-env-linux:${normal} Linking ${objcopy} to /usr/bin/x86_64-elf-objcopy."
    sudo ln -s $objcopy x86_64-elf-objcopy

    strip = $(which strip)
    echo "${bold}install-env-linux:${normal} Linking ${strip} to /usr/bin/x86_64-elf-strip."
    sudo ln -s $strip x86_64-elf-strip
