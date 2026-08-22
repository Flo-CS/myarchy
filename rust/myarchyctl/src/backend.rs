use anyhow::Result;
use myarchy_core::brightness::BrightnessCtl;
use myarchy_core::compositor::{CompositorCtl, Monitor};
use myarchy_core::cursor::CursorCtl;
use myarchy_core::error::AppError;
use myarchy_core::nightlight::NightLightCtl;
use myarchy_core::notify::{NotifierCtl, OsdCtl};
use myarchy_core::wallpaper::WallpaperCtl;
use myarchy_sys::backlight::Backlight;
use myarchy_sys::ddc::Ddc;
use myarchy_sys::{hyprctl, hyprcursor, hyprpaper, hyprsunset, notify_send, swayosd};

pub fn compositorctl() -> impl CompositorCtl {
    hyprctl::HyprctlCli
}

pub fn nightlightctl() -> impl NightLightCtl {
    hyprsunset::Hyprsunset
}

pub fn wallpaperctl() -> impl WallpaperCtl {
    hyprpaper::Hyprpaper
}

pub fn cursorctl() -> impl CursorCtl {
    hyprcursor::HyprctlCursor
}

pub fn osdctl() -> impl OsdCtl {
    swayosd::SwayOsd
}

pub fn notifierctl() -> impl NotifierCtl {
    notify_send::NotifySend
}

pub fn brightnessctl(name: &str, monitors: &[Monitor]) -> Result<Box<dyn BrightnessCtl>> {
    try_brightnessctl(name, monitors)?.ok_or_else(|| {
        AppError::DdcNotResponding {
            name: name.to_string(),
        }
        .into()
    })
}

pub fn try_brightnessctl(
    name: &str,
    monitors: &[Monitor],
) -> Result<Option<Box<dyn BrightnessCtl>>> {
    let Some(monitor) = monitors.iter().find(|m| m.name == name) else {
        return Ok(None);
    };

    match monitor.is_internal() {
        true => Ok(Some(Box::new(Backlight) as Box<dyn BrightnessCtl>)),
        false => Ok(Ddc::display_for(name, monitors)?.map(|display| {
            Box::new(Ddc::new(name.to_string(), display)) as Box<dyn BrightnessCtl>
        })),
    }
}
