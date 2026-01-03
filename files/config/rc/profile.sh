export TERMINAL=alacritty
export EDITOR=/usr/bin/nvim
export SYSTEMD_EDITOR=/usr/bin/nvim

export QT_QPA_PLATFORM='wayland;xcb'
export QT_QPA_PLATFORMTHEME=qt6ct
export QT_STYLE_OVERRIDE=Kvantum

export LIBVA_DRIVER_NAME=nvidia
export __GLX_VENDOR_LIBRARY_NAME=nvidia

export PATH="$HOME/.local/bin:$PATH"

if uwsm check may-start; then
	exec uwsm start hyprland.desktop
fi
