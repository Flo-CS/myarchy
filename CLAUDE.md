# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

This is a personal Arch Linux dotfiles repository ("myarchy") for managing system configuration on EndeavourOS. It uses a custom deployment system with bash scripts to symlink configuration files and manage themes.

Originally inspired by the Omarchy project but customized for personal use.

## Installation and Deployment

### Initial System Installation

```bash
# Run the main installation script
./install/install
```

The installer requires setting `MYARCHY_INSTANCE_ID` environment variable to identify the specific machine (e.g., 'desktop', 'laptop'). This is used to select machine-specific configuration files (like `hyprland.conf#desktop`).

### Deployment Commands

All custom scripts are prefixed with `myarchy-` and installed to `~/.local/bin`:

```bash
# Deploy all configuration files as symlinks
# (automatically runs during install)
./install/user/deploy-config

# Deploy bin scripts to ~/.local/bin
./install/user/deploy-bin

# Refresh all running applications after config changes
myarchy-refresh all
```

The `myarchy-refresh all` command reloads:
- Alacritty (terminal)
- Waybar (status bar)
- Mako (notification daemon)
- Hyprland (window manager)
- Swayosd (OSD server)
- GTK (triggers settings reload)
- Cursor (applies current theme's cursor)
- Bat (rebuilds cache)

## Architecture

### Directory Structure

```
myarchy/
├── bin/                    # Custom scripts (deployed to ~/.local/bin)
│   ├── appearance/         # Theme and appearance management
│   ├── apps/              # Application launchers
│   ├── packages/          # Package management utilities
│   ├── services/          # Service management
│   ├── system/            # System utilities (audio, battery, power, drives)
│   ├── tools/             # User-facing tools (menus, screenshots, etc.)
│   └── utils/             # Internal utilities (deploy, refresh, update)
├── files/                 # Configuration files to be symlinked
│   ├── config/            # Application configurations
│   ├── themes/            # Theme definitions
│   │   ├── current/       # Symlink to active theme
│   │   ├── forest-night/  # Theme directories
│   │   └── matte-ember/
│   └── wallpapers/        # Wallpaper files
└── install/               # Installation scripts
    ├── global/            # System-wide installation (requires sudo)
    │   └── files/         # Contains myarchy-lib
    └── user/              # User-specific installation
```

### Configuration Deployment System

The system uses **symlinks** rather than copying files. Configuration files in `files/config/` are symlinked to their target locations in `~/.config/` by the `deploy-config` script.

Instance-specific configs use the `#INSTANCE_ID` suffix pattern:
- `hyprland.conf#desktop` → symlinked to `~/.config/hypr/hyprland.conf` when `MYARCHY_INSTANCE_ID=desktop`
- `.bashrc#laptop` → symlinked to `~/.bashrc` when `MYARCHY_INSTANCE_ID=laptop`

### Theme System

Themes are located in `files/themes/` with a `current` symlink pointing to the active theme.

Each theme directory contains theme-specific configurations for:
- alacritty, bat, btop, fzf, gum, gtk, hypr, kvantum, mako, nvim, rofi, starship, swayosd, waybar
- `palette.txt` defining theme colors
- `preferred-wallpaper` file storing the wallpaper name for this theme
- `cursor` file defining cursor theme name and size for Hyprland
- GTK theme settings in `gtk/gtk-3.0/settings.ini` and `gtk/gtk-4.0/settings.ini` (includes cursor theme)

Theme management:
```bash
myarchy-theme list                        # List available themes
myarchy-theme get                         # Get current theme
myarchy-theme set <name>                  # Switch theme (also applies preferred wallpaper)
myarchy-theme add <new> <base>            # Create new theme from existing
```

Wallpaper management:
```bash
myarchy-wallpaper list                    # List wallpapers
myarchy-wallpaper get                     # Get current wallpaper
myarchy-wallpaper set <name>              # Set wallpaper
myarchy-wallpaper apply-preferred         # Apply theme's preferred wallpaper
```

Cursor management:
```bash
myarchy-cursor list                         # List available cursor themes
myarchy-cursor apply-preferred              # Apply theme's preferred cursor
myarchy-cursor set <name> <size>            # Set cursor (also saves to theme)
myarchy-cursor save-preferred <name> <size> # Save cursor to theme without applying
```

### myarchy-lib Utility

The `myarchy-lib` script (deployed to `/usr/local/bin/`) provides core utilities:

- `robust-link <target> <link_name>` - Creates symlinks with automatic backup
- `robust-copy <source> <target>` - Copies files with automatic backup
- `robust-insert-with-marker <path> <marker> <content>` - Inserts/updates content in files with markers
- `backup-file <path>` - Creates timestamped backup in `/var/backups/`
- `render-template <template> <output>` - Renders Jinja2 templates (uses `j2` command)

All operations automatically backup existing files to `/var/backups/` with timestamps before making changes.

## Development Workflow

### Modifying Configurations

1. Edit files in `files/config/` or `files/themes/current/`
2. Changes are immediately reflected (files are symlinked)
3. Run `myarchy-refresh all` to reload applications

### Modifying Scripts

1. Edit scripts in `bin/`
2. Run `./install/user/deploy-bin` to update `~/.local/bin`
3. Scripts become immediately available

### Template Files

Some configs use Jinja2 templates (`.j2` extension):
- `starship.toml.j2` → rendered to `starship.toml`
- `mako/config.ini.j2` → rendered to `mako/config`

Templates are rendered during `deploy-config` using `myarchy-lib render-template`.

## Key Technologies

- **Window Manager**: Hyprland (Wayland compositor)
- **Terminal**: Alacritty
- **Shell**: Bash with Starship prompt
- **Editor**: Neovim (extensive Lua configuration in `files/config/nvim/`)
- **Bar**: Waybar
- **Launcher**: Rofi
- **Notifications**: Mako
- **Package Manager**: pacman (Arch Linux)

## Important Notes

- All custom scripts must be prefixed with `myarchy-` to avoid name collisions
- The repository is tracked in git but deployed to `~/.local/share/myarchy`
- Never edit files in `~/.config/` directly - edit source files in `files/config/` instead
- Instance-specific files require `MYARCHY_INSTANCE_ID` to be set
- Backups are automatically created in `/var/backups/` before any destructive operations
