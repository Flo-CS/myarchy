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
| `MOD + W` | Center a lone tiled window, on the focused screen if it is ultrawide |
| `XF86MonBrightnessUp/Down` | Brightness on the **focused** screen, internal or external |

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

## Duplicate

Screens do **not** have to share a resolution, and none of them is forced off its native mode.
Hyprland renders the anchor once and copies that texture to each mirror, scaling by the smaller of
the two axis ratios and centering the result — so a mismatched aspect gets black bars rather than a
stretched picture. Each mirror keeps its own mode; only the anchor's resolution is being copied.

That means the anchor decides both sharpness and how much of the panel gets used, and the two
choices pull in opposite directions here:

| Anchor | Mirror shows | Result |
| --- | --- | --- |
| Laptop `1920x1080` | Ultrawide `3440x1440` | `2560x1440` centered, 440px bars left and right. Full height, image upscaled 1.33x. |
| Ultrawide `3440x1440` | Laptop `1920x1080` | `1920x804` centered, 138px bars top and bottom. Sharp, but a third of the laptop panel is black. |

Pick the anchor with `Set As Anchor`. There is no setting that avoids the trade — the compositor
renders one image at one resolution, and everything else is a scaled copy of it. Windows makes the
same trade differently: its Duplicate mode drops both displays to a single shared mode both support,
so the higher-resolution screen stops running natively. Hyprland at least keeps every panel on its
own native mode and does the fitting in the copy.

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

The hardware brightness keys go through `myarchyctl screen brightness-step`, which picks the backlight
or DDC/CI for whichever screen has focus. They used to call `swayosd-client --brightness` directly,
which only ever drives the internal panel, so on the desktop — and on the laptop's external screen —
the keys did nothing. Internal panels are still handed to `--brightness`, which reads the real
backlight; external ones go through `--custom-progress` instead, since `--brightness` only knows
`brightnessctl`/pulseaudio devices.

DDC/CI is slow by design: ~0.5s for a write, plus whatever ramp the monitor's own firmware applies,
so a brightness change takes 1–2 seconds to land. `ddcutil detect` costs another ~0.4s, so the
connector → display map is cached in `$XDG_RUNTIME_DIR`, keyed on the connected screens so plugging
one in invalidates it.

That latency is also why external steps don't read the display before showing the OSD: a
`ddcutil getvcp` round-trip on every key press would make the bar itself lag. `myarchyctl screen`
instead keeps the last value it wrote per display in `$XDG_RUNTIME_DIR`, computes and shows the new
target from that alone, and only then hands the write off to `queue_brightness_apply`. That function
`flock -n`s a per-display lock file; if a write is already in flight, the press just updates the
target file and returns instead of stacking up another `ddcutil` call. The one worker that holds the
lock keeps re-reading the target file and re-applying until it matches what it last wrote, so a burst
of key presses converges on one trailing write instead of one write per press — the bar keeps up
instantly, the hardware catches up in the background. `set_brightness` writes that same cache on
every successful call, including from the menu, so the two paths can't drift apart.

The worker itself writes nothing to that cache — it calls the cache-free `ddc_setvcp` in a loop and
tracks what it applied in a local variable instead. It has to: its `ddcutil setvcp` blocks for the
full 1–2s DDC/CI round trip, and if it wrote the target file with the value it started that call
with, a burst of presses landing during the wait would get overwritten back down to that stale value
the moment the call finally returned — visible as the bar snapping backward toward the real monitor
brightness mid-burst.

## Commands

```
myarchyctl display list | list-modes <name>
                extend <left|right|above|below>
                place <name> <left-of|right-of|above|below> <ref>
                mirror | only <name> | enable/disable/toggle <name>
                set-mode <name> <mode> | set-scale <name> <scale>
                primary <name> | anchor
                save | apply | auto | reset

myarchyctl screen  brightness-monitors | brightness-get <name> | brightness-set <name> <pct>
                brightness-step <+-pct> [name] | brightness-list
                nightlight-get|-set <pct>|-off|-toggle|-list
```

## Hyprland gotchas found the hard way

- **`hyprctl keyword` does nothing on a Lua config.** It prints
  `keyword can't work with non-legacy parsers. Use eval.` and still **exits 0**. Everything must go
  through `hyprctl eval` + `hl.monitor{}`. This also silently broke `myarchy-toggle-centered-mode`.
- **`hyprctl eval` always answers `ok`**, even for `return 1+1` — it has no return channel and
  reports success for a rule the compositor rejected, so checking its output only ever caught
  *syntax* errors. Everything goes through `hyprctl repl` instead, which prints the value and
  exits non-zero on error.
- **`mirror` and `disabled` are sticky.** A monitor rule that omits them keeps the previous value, so
  leaving mirror mode or re-enabling a screen needs them spelled out every time.
- **`mirrorOf` reports a monitor id, not always a name.**
- **The catch-all `hl.monitor({ output = "", position = "auto" })` re-flows any monitor you don't
  mention**, so moving one screen drags the others unless they are pinned at their current position.
- **Rule application is asynchronous.** Reading `hyprctl monitors` straight after a write returns
  the state you just replaced. `settle()` waits for two identical readings rather than sleeping a
  guessed interval, and everything is saved from the settled snapshot.
- **Your own changes fire the hotplug hooks.** This used to need a two-second debounce, which also
  swallowed genuine events. Now `apply_profile` does nothing when the layout already matches the
  profile, so the re-entrant `auto` those hooks trigger simply finds nothing to do and stops.
- **Workspaces are global**: each has a home screen, fixed where it was first opened.
  `focus({ on_current_monitor = true })` was tried and dropped — it *swaps* the two screens'
  workspaces, so switching on one screen changes what the other shows.
- **`misc:initial_workspace_tracking = 2`** sends every later window of a process back to the
  workspace it was launched on (this is why new Firefox windows kept reappearing next to the first).
  `1` (single-shot, the default) is almost always what you want.
- **Disabling a monitor doesn't move its workspaces off it** (hyprwm/Hyprland#5052). `MOD+<n>` then
  does nothing for a workspace stranded on the now-off screen — waybar flickers but the switch never
  happens, since there is no active output to show it on. `disable_monitor`/`only` in
  `myarchyctl display` now call `moveworkspacetomonitor` on every workspace living on a screen before
  disabling it, onto the screen that stays on.

## Why the engine is bash and not Lua

Measured against Hyprland 0.56.2. The config is already Lua, so moving the layout engine into it
looked obvious. It is not, for one reason:

- **`HL.Monitor` has no `disabled` field**, and `hl.get_monitors()` has no `all` variant
  (`/usr/share/hypr/stubs/hl.meta.lua`, the `HL.Monitor` class and the `HL.API` field list).
  Restoring a profile means mapping a saved description onto a screen that is currently **off**,
  and `hyprctl monitors all -j` is the only thing that can see one. Lua would have to track
  disabled outputs by hand from `monitor.added`/`removed` — bookkeeping in place of a query.

The rest of the Lua API is genuinely nicer and is used where it fits:

- **`eval`/`repl` share the config's Lua state.** `hyprctl eval 'X = 42'` then
  `hyprctl repl 'return X'` prints `42`. `myarchy-toggle-centered-mode` relies on this: it
  builds a workspace rule per screen, keeps the handles in a global, and flips `set_enabled()` on
  later presses rather than redeclaring the rule.
- **`monitor.layout_changed`** exists as a settle signal, but only inside the compositor. From
  outside, `settle()` polling for two identical readings is the equivalent.
- **Not a speed argument.** `myarchyctl display list` 9ms, `anchor` 18ms, Lua equivalent 5ms.
- `myarchyctl screen` stays bash regardless: brightnessctl, ddcutil and hyprsunset are external
  processes with no compositor state.

## Where settings live

Two stores, and the split matters:

- **Per-machine constants, in git** — `config/hypr/hyprland.lua#<instance>` holds `scale`,
  `main_mod`, `sensitivity`. These describe the machine and belong in review. A screen's own
  properties do not: centering is derived from the focused monitor, so plugging the ultrawide into
  the laptop works without touching the profile.
- **Ad-hoc layout, in runtime state** — `~/.local/state/myarchy/display/`. Where you dragged a
  screen is not a reviewable decision, and it changes whenever something is plugged in.

The Hyprland config is deliberately *not* a layout store. `install/user/deploy-config` symlinks
`config/hypr/*` into `~/.config`, so a program that rewrote its own config at runtime would be
writing into the git working tree — every scale nudge would show up as a dirty file, on every
machine. That also rules out `nwg-displays`, which writes `monitors.conf` in hyprlang that a Lua
config never reads.

## The compositor seam

`myarchyctl display` is laid out in layers: **backend**, **model**, **state**, **engine**,
**frontend**. Only the backend block knows it is talking to Hyprland — `mon_snapshot`,
`mon_apply`, `mon_disable`, `mon_reload` and their two helpers. Everything below it works on a
snapshot of JSON and would survive a move to another compositor; porting means rewriting that one
block. `grep -n hyprctl rust/myarchyctl` should only ever hit inside it.

Each command reads **one** snapshot and passes it down. That is not only cheaper than the old
per-helper queries — it means every decision within a command is made against the same view,
instead of re-reading a compositor that may be mid-transition.

kanshi and shikane were considered: they do description-keyed profile switching on hotplug over
`zwlr_output_manager_v1`, which Hyprland does implement. They were not adopted because Hyprland
drops disabled heads from that protocol's output list, which is exactly what profile restore
needs, and because they cover only the automatic half — `extend`, `place`, `only` and the DDC
brightness menu would still be ours, against a second config language.

## Waybar

Each bar marks the workspaces living on its own screen via `#workspaces button.hosting-monitor`
(`workspace-monitor == waybar-monitor`), underlined in `@accent`. `border-radius: 0` on the buttons
is required — the GTK theme's default button radius curls the underline otherwise.

`persistent-workspaces` keyed by workspace with an empty output list forces those workspaces onto
*every* bar, which is why slots 1–5 always show.

The waybar theme template now also exposes `@accent` from the palette.
