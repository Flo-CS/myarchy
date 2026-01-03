# WIKI

## ISO, USB, and Installation

### Ventoy

Use Ventoy to create a bootable USB drive allowing to boot multiple ISO files from the same USB drive.

Note that the USB drive can still be used to store files other tha ISO files. Most commonly files to set up the system after installation.
They should be placed in a directory with a file named .ventoyignore.

## Git authentication

Storing Git credentials is way more complicated than I thought, there is some options:

- Github CLI (not working like I want)
- Git Credential Manager (designed for Windows so complicated on Linux)
- Git cache, but each time cache expires we should generate new token, because Github does not allow usage of password to login
- Git plain text, not very secure, even if it's a token

The only simple and secure solution I found is using git-credential-oauth

INFO: To learn more [https://git-scm.com/doc/credential-helpers] and [https://git-scm.com/docs/gitcredentials]

## Common issues

### No sound

Try to install the sof-firmware package. sof-firmware is a collection of open source linux firmwares for commercial hardwares like Intel sound chips

### Unable to compile Hyprland plugins

I had problem with version 0.48.1 of Hyprland, I wansn't able to compile plugins, there is two reasons:

First, when running `hyprpm add https://github.com/hyprwm/hyprland-plugins`, perhaps the plugins are not compatible with the current version of Hyprland,
so you need to specify a git rev, for example `hyprpm add https://github.com/hyprwm/hyprland-plugins v0.48.0`.

Second, all the *-devel packages for the hyprland dependencies must be installed.

### Black screen just after the grub menu

#### Option 1: Monitor issue (FreeSync/HDMI)

Solution 1: disable FreeSync in the monitor settings and activate the HDMI compatibility mode (if available).
Solution 2: use DisplayPort instead of HDMI. But be careful, because FreeSync can still be enabled for DisplayPort, and it can cause different issues, like extreme flickering or screen tearing, so disable it also.

=> Seems to be related to: [https://wiki.archlinux.org/title/Variable_refresh_rate]

### Unable to launch KeepassXC AppImage

The app image of KeepassXC is only "compiled" (do we say that?) for QT platform plugin xcb.
So even QT supports wayland, it will not launch if the QT_QPA_PLATFORM env variable has been set to `wayland`. One way to fix this is to set it to `wayland;xcb`

Also, if a warning appears on lauching about a invalid style override, the QT_STYLE_OVERRIDE variable can be responsible if it's set to `Kvantum`, but I think the error can be safely ignored.

### Bluetooth devices need to be repaired every system change on dual boot

See [https://github.com/x2es/bt-dualboot]
