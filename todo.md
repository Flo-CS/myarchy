# TODO

- Change GTK and QT themes

## Actions list

- [ ] Strange purple big squared artifact when changing theme with multiple screens
      Fix is in but unverified: changing theme no longer re-applies every monitor rule, which was
      forcing a mode-set across both GPUs. Plug both screens and change theme to confirm. If it
      survives, it is buffers crossing between the Intel and NVIDIA cards - add AQ_DRM_DEVICES,
      see wiki/troubleshooting.md.
- [ ] Plymouth splash lands on a single screen, and off-centre, with two screens attached
      Fix is in but unverified: the DRM drivers were missing from the initramfs, so plymouth came
      up on a fallback framebuffer. install/global/config-boot-splashscreen now writes a
      force_drivers line. Re-run that step with MYARCHY_FORCE=1 and reboot to confirm.
- [ ] Build a smart dotfiles manager
- [ ] Make a tool to pin a window and move it on all screen (floating or not)
