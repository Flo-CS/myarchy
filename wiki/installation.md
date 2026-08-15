# Installation

## Ventoy

Use Ventoy to create a bootable USB drive allowing to boot multiple ISO files from the same USB drive.

Note that the USB drive can still be used to store files other than ISO files. Most commonly files to set up the system after installation.
They should be placed in a directory with a file named .ventoyignore.

## Full reinstall

### 1. Base system

Install EndeavourOS from the ISO. The install scripts assume:

- **EndeavourOS specifically**, not plain Arch — `config-system-backups` and `config-boot-splashscreen` rely on dracut and `dracut-rebuild`
- **btrfs root**, or snapper and grub-btrfs will fail
- **UEFI + GRUB**, since `/boot/grub/grub.cfg` is rewritten
- the same username as before, and a working network

Pick any desktop in the installer; whatever login manager it enables gets replaced by greetd.

### 2. Run the installer

```sh
git clone https://github.com/Flo-CS/config.dotfiles ~/.local/share/myarchy
~/.local/share/myarchy/install/install
```

The path matters — `preflight` refuses to run from anywhere else, because `deploy-config` and the
systemd user units hardcode it.

You are asked once for the instance ID (`desktop` or `laptop`), which selects every `#<instance>`
variant in `config/`. It can also be passed ahead of time:

```sh
MYARCHY_INSTANCE_ID=desktop ~/.local/share/myarchy/install/install
```

### 3. Reboot

greetd only takes over on reboot, and so do plymouth and the GRUB changes.

### 4. Check

- greetd/tuigreet appears, and logging in starts Hyprland
- waybar, mako, rofi and alacritty are themed
- `myarchy-menu` opens
- `myarchy-theme set matte-ember` re-renders and applies
- `sudo ufw status` is active, and firewalld is gone
- `snapper -c root list` works
- `find ~/.config -xtype l` prints nothing

Then work through [manual-restore.md](manual-restore.md) for what the repo does not install.

To rehearse all of this first, see [vm-testing.md](vm-testing.md).

## Resuming a failed install

Every step is idempotent, so re-running `install/install` is always safe. Completed steps are
recorded in `~/.local/state/myarchy/install-steps` and skipped on the next run, and the full output
is appended to `~/.local/state/myarchy/install.log`. The file is deleted once a run finishes, so
`myarchy-update all` still replays everything.

To force a step that was already recorded:

```sh
MYARCHY_FORCE=1 ~/.local/share/myarchy/install/install
```
