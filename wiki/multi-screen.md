# Multi-screen

Screen layout, brightness and night light, driven from `myarchy-menu`.

## Keybindings

| Key | Action |
| --- | --- |
| `MOD + P` | Display menu (opens `myarchy-menu` at the Display submenu) |
| `MOD + M` | Focus the next monitor (wraps) |
| `MOD + SHIFT + M` | Move the current workspace to the next monitor |
| `MOD + H/J/K/L` | Focus; crosses to the neighbouring screen at the layout edge |
| `MOD + SHIFT + H/J/K/L` | Move window; same edge behaviour |
| `MOD + X` / `MOD + SHIFT + X` | Scratchpad toggle / send window to it (moved off `P`) |
| `MOD + W` | Toggle ultrawide single-window centering |

The edge crossing is Hyprland's `binds:window_direction_monitor_fallback`, which is on by default;
it is set explicitly in `hyprland.lua` only as documentation.

**Scratchpad on another screen:** no extra keybind. Focus the screen you want (`MOD + M`), press
`MOD + X` — the special workspace follows the focused monitor.

## Menu

```
Display
  Extend            → Right / Left / Above / Below      (all screens at once)
  Duplicate                                             (others mirror the primary)
  Only <screen>                                         (disable the rest)
  Monitors → <screen>
      Position      → left-of / right-of / above / below → <reference screen>
      Toggle        (on/off)
      Set As Anchor
      Resolution    (modes reported by the screen)
      Scale         (0.5 … 3, or type any value)
  Brightness        → <screen> → 10 / 25 / 50 / 75 / 100 %   applies live, menu stays open
  Night Light       → Off / 6500K … 2700K                applies live, menu stays open
  Reset             (drop the saved layout, reload config)
```

`Set As Anchor` picks which screen **Extend** and **Duplicate** build around: Extend puts the anchor
at `0x0` and lays the others out beside it, Duplicate makes the others mirror it. Wayland has no
notion of a primary display, so it affects nothing else — not where windows or bars open. The
Monitors list marks the current one with `(anchor)`.

`Position` reorders the screens along one axis and lays them out edge to edge, so they can never
overlap. `Extend` arranges everything in one direction at once.

Brightness and Night Light apply immediately and leave the menu open, so you can step through the
levels and watch the screen. That uses the `apply` helper — **a row's command cannot contain `;`**.
`myarchy-menu` runs a row via `$ROFI_INFO` unquoted, which word-splits but never re-parses shell
syntax, so `cmd; reshow` passes `;` along as a literal argument instead of running two commands.

## How layouts are remembered

`~/.local/state/myarchy/display/<key>.tsv`, where `<key>` is a hash of the connected screens'
**descriptions** (make/model/serial), because `DP-1`/`DP-3` renumber across re-plugs.

Every mutating command snapshots the result, so the layout for a given set of screens is always the
last one you set. It is re-applied on `monitor.added` / `monitor.removed`, on `hyprland.start`, and on
`config.reloaded` (so `myarchy-refresh` no longer loses your positions). A new combination of screens
extends right and sends a notification.

Brightness and night light are **not** part of this. Night light is compositor-wide and
session-scoped. Brightness is per screen: internal panels through the kernel backlight
(`brightnessctl`), external ones over DDC/CI (`ddcutil`), matched up by the DRM connector name that
`ddcutil detect` reports — the same name Hyprland uses. Screens that don't answer DDC/CI simply don't
appear in the Brightness menu.

DDC/CI needs `ddcutil` and the `i2c-dev` module; `install/global/config-ddc` handles both. No `i2c`
group membership is required — ddcutil's udev rule tags only the display-controller buses
(`ATTRS{class}=="0x03*"`) with `uaccess`, which grants the locally logged-in user an ACL on exactly
those. Joining the `i2c` group instead would hand out read/write on *every* I2C bus, SMBus included,
where a stray write can corrupt RAM SPD data.

Run `install/global/config-ddc` once; it writes `/etc/modules-load.d/i2c-dev.conf` so the module
comes back after a reboot. Without it DDC works until the next boot and then the external screen
quietly disappears from the Brightness menu.

Laptop panels report `Invalid display ... Laptop displays do not support DDC/CI`, which is expected —
they go through `brightnessctl` instead.

DDC/CI is slow by design: ~0.5s for a write, plus whatever ramp the monitor's own firmware applies,
so a brightness change takes 1–2 seconds to land. `ddcutil detect` costs another ~0.4s, so the
connector → display map is cached in `$XDG_RUNTIME_DIR`, keyed on the connected screens so plugging
one in invalidates it. This is why there is no live brightness *slider* for external screens.

## Commands

```
myarchy-display list | list-modes <name>
                extend <left|right|above|below>
                place <name> <left-of|right-of|above|below> <ref>
                mirror | only <name> | enable/disable/toggle <name>
                set-mode <name> <mode> | set-scale <name> <scale>
                primary <name> | anchor
                save | apply | auto | reset

myarchy-screen  brightness-monitors | brightness-get <name> | brightness-set <name> <pct>
                brightness-list
                nightlight-get|-set <kelvin>|-off|-toggle|-list
```

## Hyprland gotchas found the hard way

- **`hyprctl keyword` does nothing on a Lua config.** It prints
  `keyword can't work with non-legacy parsers. Use eval.` and still **exits 0**. Everything must go
  through `hyprctl eval` + `hl.monitor{}`. This also silently broke `myarchy-toggle-ultrawide-center`.
- **`mirror` and `disabled` are sticky.** A monitor rule that omits them keeps the previous value, so
  leaving mirror mode or re-enabling a screen needs them spelled out every time.
- **`mirrorOf` reports a monitor id, not always a name.**
- **The catch-all `hl.monitor({ output = "", position = "auto" })` re-flows any monitor you don't
  mention**, so moving one screen drags the others unless they are pinned at their current position.
- **Rule application is asynchronous.** Reading `hyprctl monitors` straight after an `eval` returns
  the state you just replaced — wait for it to settle before saving.
- **Your own changes fire the hotplug hooks**, so anything that re-applies a saved profile must be
  debounced or it will undo what you just asked for.
- **Workspaces are global**: each has a home screen, fixed where it was first opened.
  `focus({ on_current_monitor = true })` was tried and dropped — it *swaps* the two screens'
  workspaces, so switching on one screen changes what the other shows.
- **`misc:initial_workspace_tracking = 2`** sends every later window of a process back to the
  workspace it was launched on (this is why new Firefox windows kept reappearing next to the first).
  `1` (single-shot, the default) is almost always what you want.

## Waybar

Each bar marks the workspaces living on its own screen via `#workspaces button.hosting-monitor`
(`workspace-monitor == waybar-monitor`), underlined in `@accent`. `border-radius: 0` on the buttons
is required — the GTK theme's default button radius curls the underline otherwise.

`persistent-workspaces` keyed by workspace with an empty output list forces those workspaces onto
*every* bar, which is why slots 1–5 always show.

The waybar theme template now also exposes `@accent` from the palette.
