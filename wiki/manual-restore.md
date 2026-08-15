# Manual restore

What `install/install` does **not** bring back. Generated from `pacman -Qqe` against
`install/global/install-packages`, plus the state that lives outside the repo.

## Secrets and keys

Nothing here is in git, and several things silently misbehave without it.

- `~/.ssh/` — `.bashrc` runs `eval "$(keychain --eval)"`, which prompts or warns without keys
- GitHub credentials — `install-idle-inhibitor` clones a private repo and only warns if it cannot, so run `myarchy-update all` again once `git-credential-oauth` has authenticated
- age / sops keys — `SOPS_EDITOR` is configured but the recipients are not
- `~/.cargo/` if you use rustup rather than the `rust` package
- syncthing device IDs and folder config (`default.target.wants/syncthing.service` is enabled by the repo, but its identity is not)
- browser profiles

## Not reproducible from the repo

- `config/nvim/lazy-lock.json` is gitignored, so nvim plugin versions are not pinned. First launch resolves whatever is current.
- `~/.local/share/applications` entries created by `myarchy-webapp` and `myarchy-appimage` — `deploy-applications` skips files that already exist and never tracks them.
- dconf/gsettings state beyond what `myarchy-theme` writes.

## Packages

Drivers, firmware and the Xorg/NetworkManager stack come from the EndeavourOS ISO — do not reinstall
those by hand.

Needed by things the repo ships:

```sh
yay -S --needed localsend-bin        # myarchy-share
yay -S --needed granite-gtk-theme    # gtk-theme=Granite in themes/matte-ember/vars.txt
sudo pacman -S --needed qt6ct        # QT_QPA_PLATFORMTHEME=qt6ct in config/rc/profile.sh
sudo pacman -S --needed breeze       # gtk-icon-theme=breeze
```

Note: `granite-gtk-theme`, `qt6ct` and `breeze` are **not installed today either**, so those three
settings are currently no-ops. Either install them or change `vars.txt` / `profile.sh`.

Discretionary apps, restore as needed:

```sh
sudo pacman -S --needed 7zip ansible ansible-lint aspell aspell-fr blender diff-so-fancy duf \
  ffmpegthumbnailer firefox-i18n-fr glances glow gparted gst-libav gst-plugins-bad \
  gst-plugins-ugly impala jdk-openjdk kdeconnect libdvdcss meld nano-syntax-highlighting nmap \
  nnn noto-fonts-cjk noto-fonts-extra openssh pacman-contrib pkgfile plocate python-pipx ranger \
  rust-analyzer steam swaync ttf-bitstream-vera ttf-dejavu ttf-liberation ttf-opensans unrar \
  wget wireguard-tools

yay -S --needed android-studio downgrade jetbrains-toolbox ollama opencode-bin \
  spotify-launcher visual-studio-code-bin
```

## Deliberately dropped

- `wofi` — config and package both removed; rofi is the launcher
- `firewalld` — `config-firewall` now disables and removes it so it stops fighting ufw
- `plymouth-theme-circle-hud-git` — installed on the old system, but the repo installs and selects `plymouth-theme-spinner-alt-git`
