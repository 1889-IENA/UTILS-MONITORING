#!/bin/bash


clear


sudo apt purge --auto-remove cargo rustup


sudo rustup toolchain uninstall stable


read -n 1 -s -r -p "Done"
