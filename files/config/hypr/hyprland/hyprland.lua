local profile = PROFILE

hl.monitor({
	output = "",
	mode = "preferred",
	position = "auto",
	scale = "auto",
})

hl.on("hyprland.start", function()
	hl.exec_cmd("uwsm app -- waybar >/dev/null 2>&1 &")
	hl.exec_cmd("hyprpaper")
	hl.exec_cmd("hypridle")
	hl.exec_cmd("systemctl --user start hyprpolkitagent")
	hl.exec_cmd("uwsm app -- udiskie")
	hl.exec_cmd("uwsm app -- mako >/dev/null 2>&1 &")
	hl.exec_cmd("uwsm app -- swayosd-server >/dev/null 2>&1 &")
	hl.exec_cmd("myarchy-cursor apply-preferred")
	hl.exec_cmd("gnome-keyring-daemon --start --components=secrets,pkcs11")
end)

hl.config({
	dwindle = {
		preserve_split = true,
	},

	misc = {
		disable_hyprland_logo = true,
		force_default_wallpaper = 0,
		allow_session_lock_restore = true,
		initial_workspace_tracking = 2,
		key_press_enables_dpms = true,
		disable_autoreload = true,
		enable_swallow = true,
		swallow_regex = "^(Alacritty)$",
		vrr = 1,
	},

	binds = {
		hide_special_on_workspace_change = true,
		workspace_back_and_forth = true,
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

if profile.ultrawide then
	-- Center the lone tiled window on ultrawide monitors.
	-- `w[t1]` selector: a workspace containing exactly one tiled window.
	hl.workspace_rule({
		workspace = "w[t1]",
		gaps_out = { top = 20, right = 600, bottom = 20, left = 600 },
	})
	hl.bind(profile.main_mod .. " + W", hl.dsp.exec_cmd("myarchy-toggle-ultrawide-center"))
end

require("hyprland.keybindings")
require("hyprland.style")
require("hyprland.theme")
require("hyprland.apps")
