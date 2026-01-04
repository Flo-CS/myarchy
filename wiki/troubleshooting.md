# Troubleshooting

## No sound

Try to install the sof-firmware package. sof-firmware is a collection of open source linux firmwares for commercial hardwares like Intel sound chips

## Unable to compile Hyprland plugins

I had problem with version 0.48.1 of Hyprland, I wansn't able to compile plugins, there is two reasons:

First, when running `hyprpm add https://github.com/hyprwm/hyprland-plugins`, perhaps the plugins are not compatible with the current version of Hyprland,
so you need to specify a git rev, for example `hyprpm add https://github.com/hyprwm/hyprland-plugins v0.48.0`.

Second, all the *-devel packages for the hyprland dependencies must be installed.

## Black screen just after the grub menu

### Option 1: Monitor issue (FreeSync/HDMI)

Solution 1: disable FreeSync in the monitor settings and activate the HDMI compatibility mode (if available).
Solution 2: use DisplayPort instead of HDMI. But be careful, because FreeSync can still be enabled for DisplayPort, and it can cause different issues, like extreme flickering or screen tearing, so disable it also.

=> Seems to be related to: [https://wiki.archlinux.org/title/Variable_refresh_rate]

## Unable to launch KeepassXC AppImage

The app image of KeepassXC is only "compiled" (do we say that?) for QT platform plugin xcb.
So even QT supports wayland, it will not launch if the QT_QPA_PLATFORM env variable has been set to `wayland`. One way to fix this is to set it to `wayland;xcb`

Also, if a warning appears on lauching about a invalid style override, the QT_STYLE_OVERRIDE variable can be responsible if it's set to `Kvantum`, but I think the error can be safely ignored.

## Bluetooth devices need to be repaired every system change on dual boot

See [https://github.com/x2es/bt-dualboot]
