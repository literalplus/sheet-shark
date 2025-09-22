#!/usr/bin/env bash

cargo install --locked --path .

DESKTOP_FILES=$HOME/.local/share/applications
ICON_FILES=$HOME/.local/share/icons/hicolor
CONFIG_DIR=$HOME/.config/sheet-shark

mkdir -v -p $DESKTOP_FILES
mkdir -v -p $ICON_FILES
mkdir -v -p $CONFIG_DIR

cp -v install/sheet-shark.desktop $DESKTOP_FILES/sheet-shark.desktop
cp -v install/sheet-shark_48.png $ICON_FILES/48x48/apps/sheet-shark.png
cp -v install/sheet-shark.png $ICON_FILES/512x512/apps/sheet-shark.png
cp -v --update=none install/config.yaml $CONFIG_DIR/config.yaml
