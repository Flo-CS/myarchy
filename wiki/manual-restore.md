# Manual restore

## Secrets and keys

- Add necessary private/public ssh keys in .ssh
- Add necessary sops/age keys in 

Nothing here is in git, and several things silently misbehave without it.

- age / sops keys — `SOPS_EDITOR` is configured but the recipients are not
- `~/.cargo/` if you use rustup rather than the `rust` package
- syncthing device IDs and folder config (`default.target.wants/syncthing.service` is enabled by the repo, but its identity is not)
- browser profiles

## Not reproducible from the repo

- `config/nvim/lazy-lock.json` is gitignored, so nvim plugin versions are not pinned. First launch resolves whatever is current.
- `~/.local/share/applications` entries created by `myarchy-webapp` and `myarchy-appimage` — `deploy-applications` skips files that already exist and never tracks them.
- dconf/gsettings state beyond what `myarchy-theme` writes.

## Packages

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

