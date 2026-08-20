local profile = PROFILE

-- Only reached by a screen `myarchyctl display` has no profile for
hl.monitor({
	output = "",
	mode = "preferred",
	position = "auto",
	scale = "auto",
})

local state = os.getenv("XDG_STATE_HOME") or (os.getenv("HOME") .. "/.local/state")
local saved_monitors = loadfile(state .. "/myarchy/display/current.lua")
if saved_monitors then
	saved_monitors()
end

hl.on("hyprland.start", function()
	hl.exec_cmd("myarchy-cursor apply-preferred")
	hl.exec_cmd("myarchyctl display auto")
end)

-- A dock fires one event per output, so coalesce the burst into one apply.
local function on_monitors_changed()
	hl.timer(function()
		hl.exec_cmd("myarchyctl display auto")
	end, { timeout = 300, type = "oneshot" })
end

hl.on("monitor.added", on_monitors_changed)
hl.on("monitor.removed", on_monitors_changed)

hl.config({
	dwindle = {
		preserve_split = true,
	},

	misc = {
		disable_hyprland_logo = true,
		disable_splash_rendering = true,
		force_default_wallpaper = 0,
		allow_session_lock_restore = true,
		-- 1 = single-shot; 2 sends every later window back to the launch workspace
		initial_workspace_tracking = 1,
		key_press_enables_dpms = true,
		disable_autoreload = true,
		enable_swallow = true,
		swallow_regex = "^(Alacritty)$",
		vrr = 1,
	},

	binds = {
		hide_special_on_workspace_change = true,
		workspace_back_and_forth = true,
		-- Default-on; H/J/K/L cross to the neighbouring monitor at the edge
		window_direction_monitor_fallback = true,
	},

	ecosystem = {
		no_update_news = true,
		no_donation_nag = true,
	},
})

hl.config({
	input = {
		kb_layout = "fr",
		numlock_by_default = true,
		repeat_delay = 250,
		repeat_rate = 35,

		follow_mouse = 2,
		special_fallthrough = true,
		sensitivity = profile.sensitivity,

		touchpad = {
			natural_scroll = true,
			disable_while_typing = true,
			scroll_factor = 1.2,
			drag_lock = true,
		},
	},
})

hl.bind(profile.main_mod .. " + W", hl.dsp.exec_cmd("myarchy-toggle-centered-mode"))

require("hyprland.keybindings")
require("hyprland.style")
require("hyprland.theme")
require("hyprland.apps")
