use myarchy_core::app::{Brightness, Cursor, Display, Wallpaper};
use myarchy_core::core::idle::IdleCtl;
use myarchy_core::core::nightlight::NightlightCtl;
use myarchy_core::core::notify::{NotifierCtl, OsdCtl, Silent};
use myarchy_core::host::store::{CursorFiles, LayoutFiles, WallpaperFiles};
use myarchy_core::host::theme::MyarchyTheme;
use myarchy_core::host::{brightness, hyprctl, hyprpaper, hyprsunset, idle};
use myarchy_core::host::{notify_send, swayosd};

pub struct Host {
    notifier: Box<dyn NotifierCtl>,
    osd: Box<dyn OsdCtl>,
}

impl Host {
    /// A command only reaches the user's screen when its invoker asked for it, so everything a
    /// keybinding or a hotplug hook wants to surface is opt-in and anything else talks to `Silent`.
    pub fn new(notify: bool, osd: bool) -> Self {
        Self {
            notifier: if notify {
                Box::new(notify_send::NotifySend)
            } else {
                Box::new(Silent)
            },
            osd: if osd {
                Box::new(swayosd::SwayOsd)
            } else {
                Box::new(Silent)
            },
        }
    }

    pub fn notifier(&self) -> &dyn NotifierCtl {
        &*self.notifier
    }

    pub fn display(&self) -> Display<'_> {
        Display {
            compositor: &hyprctl::Hyprctl,
            store: &LayoutFiles,
            notifier: &*self.notifier,
            settle: Default::default(),
        }
    }

    pub fn brightness(&self) -> Brightness<'_> {
        Brightness {
            compositor: &hyprctl::Hyprctl,
            source: &brightness::Backends,
            osd: &*self.osd,
        }
    }

    pub fn wallpaper(&self) -> Wallpaper<'_> {
        Wallpaper {
            backend: &hyprpaper::Hyprpaper,
            store: &WallpaperFiles,
            theme: &MyarchyTheme,
        }
    }

    pub fn cursor(&self) -> Cursor<'_> {
        Cursor {
            backend: &hyprctl::HyprctlCursor,
            store: &CursorFiles,
            theme: &MyarchyTheme,
        }
    }

    pub fn nightlight(&self) -> impl NightlightCtl {
        hyprsunset::Hyprsunset
    }

    pub fn idle(&self) -> impl IdleCtl {
        idle::IdleInhibitor
    }
}
