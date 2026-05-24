-- Common
hl.window_rule({
	name = "suppress-maximize",
	match = { class = ".*" },
	suppress_event = "maximize",
})

-- Browser
hl.window_rule({
	name = "browser-opacity",
	match = { class = "(firefox|Chromium|chrome)" },
	opacity = "1.0 override",
})

hl.window_rule({
	name = "picture-in-picture",
	match = { title = "^(Picture-in-Picture)(.*)$" },
	float = true,
	pin = true,
	keep_aspect_ratio = true,
	persistent_size = true,
})

-- IDEs
hl.window_rule({
	name = "ide-opacity",
	match = { class = "(Code|Emulator|jetbrains-studio|jetbrains-idea|jetbrains-webstorm)" },
	opacity = "1.0 override",
})

hl.window_rule({
	name = "jetbrains-toolbox-scratchpad",
	match = { class = "(jetbrains-toolbox)" },
	workspace = "special:scratchpad",
})

hl.window_rule({
	name = "emulator-style",
	match = { class = "(Emulator)" },
	border_size = 0,
	no_shadow = true,
	decorate = false,
	immediate = false,
})

-- Video players
hl.window_rule({
	name = "video-player-opacity",
	match = { class = "(mpv|vlc)" },
	opacity = "1.0 override",
})

-- File dialogs
hl.window_rule({
	name = "file-dialogs",
	match = {
		title = "^(Open File)|(Select a File)|(Choose wallpaper)|(Open Folder)|(Save As)|(Library)|(File Upload)(.*)$",
	},
	float = true,
	center = true,
})

-- KeePassXC
hl.window_rule({
	name = "keepassxc-opacity",
	match = { class = "(KeePassXC|org.keepassxc.KeePassXC)" },
	opacity = "1.0 override",
})

hl.window_rule({
	name = "keepassxc-scratchpad",
	match = { class = "(KeePassXC|org.keepassxc.KeePassXC)" },
	workspace = "special:scratchpad",
})

-- ProtonVPN
hl.window_rule({
	name = "protonvpn-scratchpad",
	match = { class = "(protonvpn-app)" },
	workspace = "special:scratchpad",
})

-- Satty (image annotation)
hl.window_rule({
	name = "satty",
	match = { class = "(com.gabm.satty)" },
	float = true,
	center = true,
})

-- Myarchy menus / apps
hl.window_rule({
	name = "myarchy-menu",
	match = { class = "^(Myarchy.)(Menu|App|BigApp)$" },
	float = true,
	center = true,
})

hl.window_rule({
	name = "myarchy-menu-size",
	match = { class = "^(Myarchy.)(Menu|App)$" },
	size = "1000 700",
})

hl.window_rule({
	name = "myarchy-bigapp-size",
	match = { class = "^(Myarchy.BigApp)$" },
	size = "1400 900",
})
