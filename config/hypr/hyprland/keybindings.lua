local profile = PROFILE

local function osd(args)
	return [[swayosd-client --monitor "$(hyprctl monitors -j | jq -r '.[] | select(.focused == true).name')" ]] .. args
end

local function osd_bind(key, args, desc, extra)
	local opts = { locked = true, repeating = true, description = desc }
	if extra then
		for k, v in pairs(extra) do
			opts[k] = v
		end
	end
	hl.bind(key, hl.dsp.exec_cmd(osd(args)), opts)
end

-- SYSTEM/POWER

hl.bind(profile.main_mod .. " + CTRL + SHIFT + escape", hl.dsp.exec_cmd("uwsm stop"))
hl.bind(profile.main_mod .. " + CTRL + L", hl.dsp.exec_cmd("uwsm app -- myarchy-session lock"))

-- APP LAUNCHERS

hl.bind(profile.main_mod .. " + Q", hl.dsp.exec_cmd("uwsm app -- alacritty"))
hl.bind(
	profile.main_mod .. "+ SHIFT" .. " + E",
	hl.dsp.exec_cmd([[uwsm app -- rofi -modi emoji -show emoji -display-emoji "Emoji"]])
)
hl.bind(profile.main_mod .. "+ SHIFT" .. " + S", hl.dsp.exec_cmd("uwsm app -- myarchy-screenshot area"))
hl.bind(profile.main_mod .. "+ SHIFT" .. " + I", hl.dsp.exec_cmd("uwsm app -- myarchy-hypr-window-info-menu"))

hl.bind(
	profile.main_mod .. " + space",
	hl.dsp.exec_cmd(
		[[uwsm app -- rofi -show combi -display-combi "▶" -modes combi -combi-modes "window,drun" -show-icons]]
	)
)
hl.bind(profile.main_mod .. "+ SHIFT" .. " + space", hl.dsp.exec_cmd("uwsm app -- myarchy-menu"))

-- WINDOW MANAGEMENT

hl.bind(profile.main_mod .. " + C", hl.dsp.window.close())
hl.bind(profile.main_mod .. " + V", hl.dsp.window.float({ action = "toggle" }))
hl.bind(profile.main_mod .. " + O", hl.dsp.window.pseudo())
hl.bind(profile.main_mod .. " + S", hl.dsp.layout("togglesplit")) -- dwindle only
hl.bind(profile.main_mod .. " + F", hl.dsp.window.fullscreen({ mode = "fullscreen" }))
hl.bind(profile.main_mod .. " + G", hl.dsp.group.toggle())
hl.bind(profile.main_mod .. "+ SHIFT" .. " + G", hl.dsp.group.lock_active({ action = "toggle" }))

-- FOCUS / MOVEMENT

hl.bind(profile.main_mod .. " + H", hl.dsp.focus({ direction = "l" }))
hl.bind(profile.main_mod .. " + L", hl.dsp.focus({ direction = "r" }))
hl.bind(profile.main_mod .. " + K", hl.dsp.focus({ direction = "u" }))
hl.bind(profile.main_mod .. " + J", hl.dsp.focus({ direction = "d" }))

hl.bind(profile.main_mod .. "+ SHIFT" .. " + H", hl.dsp.window.move({ direction = "l", group_aware = true }))
hl.bind(profile.main_mod .. "+ SHIFT" .. " + L", hl.dsp.window.move({ direction = "r", group_aware = true }))
hl.bind(profile.main_mod .. "+ SHIFT" .. " + K", hl.dsp.window.move({ direction = "u", group_aware = true }))
hl.bind(profile.main_mod .. "+ SHIFT" .. " + J", hl.dsp.window.move({ direction = "d", group_aware = true }))

-- MOUSE BINDS

hl.bind(profile.main_mod .. " + mouse:272", hl.dsp.window.drag(), { mouse = true })
hl.bind(profile.main_mod .. " + mouse:273", hl.dsp.window.resize(), { mouse = true })

-- RESIZE SUBMAP

hl.define_submap("resize", function()
	hl.bind("L", hl.dsp.window.resize({ x = 100, y = 0, relative = true }), { repeating = true })
	hl.bind("H", hl.dsp.window.resize({ x = -100, y = 0, relative = true }), { repeating = true })
	hl.bind("K", hl.dsp.window.resize({ x = 0, y = -100, relative = true }), { repeating = true })
	hl.bind("J", hl.dsp.window.resize({ x = 0, y = 100, relative = true }), { repeating = true })
	hl.bind("catchall", hl.dsp.submap("reset"))
end)
hl.bind(profile.main_mod .. " + R", hl.dsp.submap("resize"))

-- GROUPS / CYCLE

hl.bind(profile.main_mod .. " + twosuperior", hl.dsp.group.next())

hl.bind(profile.main_mod .. " + Tab", function()
	hl.dispatch(hl.dsp.window.cycle_next({ next = true }))
	hl.dispatch(hl.dsp.window.alter_zorder({ mode = "top" }))
end, { description = "Cycle to next window" })

hl.bind(profile.main_mod .. "+ SHIFT" .. " + Tab", function()
	hl.dispatch(hl.dsp.window.cycle_next({ next = false }))
	hl.dispatch(hl.dsp.window.alter_zorder({ mode = "top" }))
end, { description = "Cycle to previous window" })

-- WORKSPACES

hl.bind(
	profile.main_mod .. " + mouse_down",
	hl.dsp.focus({ workspace = "e+1" }),
	{ description = "Scroll active workspace forward" }
)
hl.bind(
	profile.main_mod .. " + mouse_up",
	hl.dsp.focus({ workspace = "e-1" }),
	{ description = "Scroll active workspace backward" }
)

local azerty = {
	"ampersand", -- 1
	"eacute", -- 2
	"quotedbl", -- 3
	"apostrophe", -- 4
	"parenleft", -- 5
	"minus", -- 6
	"egrave", -- 7
	"underscore", -- 8
	"ccedilla", -- 9
	"agrave", -- 10
}
for i, key in ipairs(azerty) do
	hl.bind(profile.main_mod .. " + " .. key, hl.dsp.focus({ workspace = i }))
	hl.bind(profile.main_mod .. "+ SHIFT" .. " + " .. key, hl.dsp.window.move({ workspace = i }))
end

-- SCRATCHPAD

hl.bind(profile.main_mod .. " + P", hl.dsp.workspace.toggle_special("scratchpad"))
hl.bind(
	profile.main_mod .. "+ SHIFT" .. " + P",
	hl.dsp.window.move({ workspace = "special:scratchpad", follow = false })
)

-- VOLUME

local vol = tostring(profile.volume_increment)
osd_bind("XF86AudioRaiseVolume", "--output-volume +" .. vol, "Volume up")
osd_bind("XF86AudioLowerVolume", "--output-volume -" .. vol, "Volume down")
osd_bind("XF86AudioMute", "--output-volume mute-toggle", "Mute")
osd_bind("XF86AudioMicMute", "--input-volume mute-toggle", "Mute microphone")

hl.bind(
	profile.main_mod .. " + XF86AudioMute",
	hl.dsp.exec_cmd("myarchy-audio switch"),
	{ locked = true, description = "Switch audio output" }
)

-- BRIGHTNESS

local bri = tostring(profile.brightness_increment)
osd_bind("XF86MonBrightnessUp", "--brightness +" .. bri, "Brightness up")
osd_bind("XF86MonBrightnessDown", "--brightness -" .. bri, "Brightness down")

-- CAPS LOCK

hl.bind(
	"CAPS + Caps_Lock",
	hl.dsp.exec_cmd(osd("--caps-lock")),
	{ locked = true, repeating = true, description = "Toggle caps lock" }
)

-- MEDIA

osd_bind("XF86AudioNext", "--playerctl next", "Next track", { repeating = false })
osd_bind("XF86AudioPause", "--playerctl play-pause", "Pause", { repeating = false })
osd_bind("XF86AudioPlay", "--playerctl play-pause", "Play", { repeating = false })
osd_bind("XF86AudioPrev", "--playerctl previous", "Previous track", { repeating = false })
