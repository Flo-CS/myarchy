pub(crate) mod icon;
pub(crate) mod theme;
pub(crate) mod xdg;

mod backlight;
mod ddc;
mod hyprctl;
mod hyprctl_cursor;
mod hyprpaper;
mod hyprsunset;
mod notify_send;
mod swayosd;

use anyhow::Result;

use crate::desktop::cursor::CursorCtl;
use crate::desktop::nightlight::NightLightCtl;
use crate::desktop::wallpaper::WallpaperCtl;
use crate::display::brightness::{BrightnessCtl, OsdCtl};
use crate::display::monitor::Monitor;
use crate::display::{CompositorCtl, NotifierCtl};
use crate::error::AppError;

use backlight::Backlight;
use ddc::Ddc;

pub(crate) fn nightlightctl() -> impl NightLightCtl {
    hyprsunset::Hyprsunset
}

pub(crate) fn wallpaperctl() -> impl WallpaperCtl {
    hyprpaper::Hyprpaper
}

pub(crate) fn cursorctl() -> impl CursorCtl {
    hyprctl_cursor::HyprctlCursor
}

pub(crate) fn compositorctl() -> impl CompositorCtl {
    hyprctl::HyprctlCli
}

pub(crate) fn osdctl() -> impl OsdCtl {
    swayosd::SwayOsd
}

pub(crate) fn notifierctl() -> impl NotifierCtl {
    notify_send::NotifySend
}

pub(crate) fn brightnessctl(
    name: &str,
    monitors: &[Monitor],
) -> Result<Box<dyn BrightnessCtl>> {
    try_brightnessctl(name, monitors)?.ok_or_else(|| {
        AppError::DdcNotResponding {
            name: name.to_string(),
        }
        .into()
    })
}

pub(crate) fn try_brightnessctl(
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
