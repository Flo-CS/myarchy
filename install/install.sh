#!/usr/bin/env bash

set -eEo pipefail

show_section() {
	gum style --border double " $1 "
}

# Define myarchy location
export MYARCHY_PATH=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)
export MYARCHY_INSTALL="$MYARCHY_PATH/install"
export PATH=$MYARCHY_PATH/bin:$PATH

show_section "Installation of base packages"
$MYARCHY_INSTALL/install-base

show_section "Deployment of system wide binaries"
$MYARCHY_INSTALL/deploy-system-bins

show_section "Configuration of myarchy"
source $MYARCHY_INSTALL/configure-myarchy

show_section "Update of system packages"
sudo pacman -Syu --noconfirm

show_section "Installation of additional packages"
$MYARCHY_INSTALL/install-additional

show_section "Installation of development packages"
$MYARCHY_INSTALL/install-dev

show_section "Setup of system backups"
$MYARCHY_INSTALL/setup-system-backups

show_section "Setup of grub"
$MYARCHY_INSTALL/setup-grub

show_section "Updating sudoers configuration"
$MYARCHY_INSTALL/configure-sudoers

show_section "Deployment of user files (config, applications, ...)"
$MYARCHY_INSTALL/deploy-user-files

show_section "Installation complete !"
gum confirm "Do you want to reboot now? It may be absolutely necessary for some programs to work correctly" && {
	sudo reboot
}

# Install
source "$OMARCHY_INSTALL/helpers/all.sh"
source "$OMARCHY_INSTALL/preflight/all.sh"
source "$OMARCHY_INSTALL/packaging/all.sh"
source "$OMARCHY_INSTALL/config/all.sh"
source "$OMARCHY_INSTALL/login/all.sh"
source "$OMARCHY_INSTALL/post-install/all.sh"
