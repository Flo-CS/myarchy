# Troubleshooting

## Unable to compile Hyprland plugins

I had problem with version 0.48.1 of Hyprland, I wansn't able to compile plugins, there is two reasons:

First, when running `hyprpm add https://github.com/hyprwm/hyprland-plugins`, perhaps the plugins are not compatible with the current version of Hyprland,
so you need to specify a git rev, for example `hyprpm add https://github.com/hyprwm/hyprland-plugins v0.48.0`.

Second, all the *-devel packages for the hyprland dependencies must be installed.

## Black screen just after the grub menu

### Monitor issue (FreeSync/HDMI)

Solution 1: disable FreeSync in the monitor settings and activate the HDMI compatibility mode (if available).
Solution 2: use DisplayPort instead of HDMI. But be careful, because FreeSync can still be enabled for DisplayPort, and it can cause different issues, like extreme flickering or screen tearing, so disable it also.

=> Seems to be related to: [https://wiki.archlinux.org/title/Variable_refresh_rate]

## Unable to launch KeepassXC AppImage

The app image of KeepassXC is only "compiled" (do we say that?) for QT platform plugin xcb.
So even QT supports wayland, it will not launch if the QT_QPA_PLATFORM env variable has been set to `wayland`. One way to fix this is to set it to `wayland;xcb`

Also, if a warning appears on lauching about a invalid style override, the QT_STYLE_OVERRIDE variable can be responsible if it's set to `Kvantum`, but I think the error can be safely ignored.

## Bluetooth devices need to be repaired every system change on dual boot

See [https://github.com/x2es/bt-dualboot]

## BTRFS and GRUB

### "Sparse file not allowed" error (or something like that)

GRUB does not support BTRFS filesystem, so it can't write to grubenv file, which can be used to store the next boot for example.

## Hybrid graphics (Intel + NVIDIA)

The laptop has two GPUs and the external ports are split across them:

```
card1  NVIDIA TU117M   DP-3, HDMI-A-3
card2  Intel i915      eDP-1, DP-1, DP-2, HDMI-A-1, HDMI-A-2
```

Check with `for d in /sys/class/drm/card*-*; do echo "$d $(cat $d/status)"; done` and
`lspci -k | grep -A3 VGA`.

### Purple/magenta rectangles when the screen is reconfigured

Blocks of solid magenta are a buffer being scanned out with the wrong tiling modifier — the
signature of buffers crossing between the two GPUs. Anything that forces a mode-set can trigger it,
and changing theme used to be one: `myarchy-theme` runs `myarchy-refresh`, which reloads the
Hyprland config, which fires `config.reloaded`, which re-applied every monitor rule. `apply_profile`
now does nothing when the layout already matches the saved profile, so a theme change no longer
mode-sets anything.

If it still happens, pin the compositor to one GPU so buffers stop crossing cards. `eDP-1` is on
Intel, so Intel goes first; the NVIDIA node stays in the list so its ports keep working:

```bash
# config/.bash_profile#laptop, next to LIBVA_DRIVER_NAME — laptop only, the
# desktop has different hardware
export AQ_DRM_DEVICES=/dev/dri/card2:/dev/dri/card1
```

It has to be in the session environment, not the Hyprland config: Aquamarine reads it when the
compositor starts, and `hl.env` only sets the environment of processes Hyprland spawns. That is
also why it sits with the other GPU variables in `.bash_profile#<instance>` rather than anywhere
under `config/hypr`.

Card numbers are not stable across boots; match on `/dev/dri/by-path/` if it moves.

### Boot or shutdown splash on the wrong screen, or drawn twice

Without the DRM drivers in the initramfs, plymouth starts on a fallback framebuffer and the real
mode-set happens late, so on two GPUs with two screens the splash lands wherever the fallback put it.
`install/global/config-boot-splashscreen` writes a `force_drivers` line into
`/etc/dracut.conf.d/plymouth.conf`, detected from the running system, and rebuilds the initramfs.

Verify with `lsinitrd | grep -E 'i915|nvidia'`. Re-run the step with `MYARCHY_FORCE=1` if the GPUs
changed.

## Grey screen with a changing sentence at the end of the session

That is Hyprland's own background, not hyprpaper. Hyprland paints `misc:background_color` over
anything no surface covers and, unless `misc:disable_splash_rendering` is set, a random splash line
at the bottom — `hyprctl splash` prints the current one. `misc:force_default_wallpaper` and
`misc:disable_hyprland_logo` only drop the image, the text is a separate switch.

hyprpaper draws the wallpaper as an ordinary layer-surface on top of that. `hyprpaper.service` is
`PartOf=graphical-session.target` and `wayland-wm@.service` is `Before=` it, so systemd stops
hyprpaper first and the compositor last, and the background shows through in between. The splash is
now off and `misc:background_color` is rendered from the theme's `surface`, so that frame is plain
theme black instead.
