use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::time::Duration;

use crate::core::compositor::{CompositorCtl, Monitor, Workspace, name_of};
use crate::core::error::UserError;
use crate::core::layout::fixtures::{laptop, ultrawide};
use crate::core::layout::{Layout, Mode, Position, Scale, Side, State};
use crate::core::notify::Silent;
use crate::core::resolution::{Resolution, Size};
use crate::core::store::LayoutStore;
use anyhow::Result;

use super::{Display, Settle};

/// Obeys the rules it is handed the way hyprland does: nothing changes until `reload`, `preferred`
/// resolves to the screen's first mode, and `auto-right` lands past whatever is already placed.
struct FakeCompositor {
    monitors: RefCell<Vec<Monitor>>,
    workspaces: RefCell<Vec<Workspace>>,
    pending: RefCell<Option<Layout>>,
    obeys: Cell<bool>,
    reloads: Cell<usize>,
    moved: RefCell<Vec<(String, String)>>,
}

impl FakeCompositor {
    fn new(monitors: Vec<Monitor>) -> Self {
        Self {
            monitors: RefCell::new(monitors),
            workspaces: RefCell::new(Vec::new()),
            pending: RefCell::new(None),
            obeys: Cell::new(true),
            reloads: Cell::new(0),
            moved: RefCell::new(Vec::new()),
        }
    }

    fn with_workspaces(self, workspaces: &[(&str, &str)]) -> Self {
        *self.workspaces.borrow_mut() = workspaces
            .iter()
            .map(|(name, monitor)| Workspace {
                name: (*name).into(),
                monitor: (*monitor).into(),
            })
            .collect();
        self
    }

    fn replug(&self, monitors: Vec<Monitor>) {
        *self.monitors.borrow_mut() = monitors;
    }

    fn at(&self, name: &str) -> (i64, i64) {
        let monitors = self.monitors.borrow();
        let m = monitors.iter().find(|m| m.name == name).unwrap();
        (m.x, m.y)
    }

    fn mode_of(&self, name: &str) -> Resolution {
        let monitors = self.monitors.borrow();
        monitors.iter().find(|m| m.name == name).unwrap().resolution
    }

    fn is_disabled(&self, name: &str) -> bool {
        self.monitors
            .borrow()
            .iter()
            .find(|m| m.name == name)
            .unwrap()
            .disabled
    }

    fn apply(&self, layout: &Layout) {
        let before = self.monitors.borrow().clone();
        let mut monitors = before.clone();

        for monitor in &mut monitors {
            let Some(screen) = layout.screens.get(&monitor.description) else {
                continue;
            };
            match &screen.state {
                State::Off => {
                    *monitor = switched_off(monitor.clone());
                }
                State::Mirroring(target) => {
                    monitor.disabled = false;
                    monitor.mirror_of = name_of(&before, target).map(str::to_string);
                }
                State::On => {
                    monitor.disabled = false;
                    monitor.mirror_of = None;
                    monitor.resolution = match screen.placement.mode {
                        Mode::Fixed(resolution) => resolution,
                        Mode::Preferred => monitor.resolutions[0],
                    };
                    monitor.scale = match screen.placement.scale {
                        Scale::Factor(v) if v > 0.0 => v,
                        _ => 1.0,
                    };
                }
            }
        }

        let laid_out: Vec<usize> = (0..monitors.len())
            .filter(|i| {
                layout
                    .screens
                    .get(&monitors[*i].description)
                    .is_some_and(|s| s.is_on())
            })
            .collect();

        let size_of = |m: &Monitor| {
            (
                (m.resolution.size.width as f64 / m.scale).round() as i64,
                (m.resolution.size.height as f64 / m.scale).round() as i64,
            )
        };

        let mut boxes: Vec<(i64, i64, i64, i64)> = Vec::new();
        let mut deferred = Vec::new();
        for i in laid_out {
            let position = layout.screens[&monitors[i].description].placement.position;
            let (width, height) = size_of(&monitors[i]);
            match position {
                Position::At(p) => {
                    monitors[i].x = p.x;
                    monitors[i].y = p.y;
                    boxes.push((p.x, p.y, width, height));
                }
                other => deferred.push((i, other, width, height)),
            }
        }
        for (i, position, width, height) in deferred {
            let (x, y) = free_spot(&boxes, position, width, height);
            monitors[i].x = x;
            monitors[i].y = y;
            boxes.push((x, y, width, height));
        }

        *self.monitors.borrow_mut() = monitors;
    }
}

fn switched_off(mut monitor: Monitor) -> Monitor {
    monitor.disabled = true;
    monitor.mirror_of = None;
    monitor.resolution = Resolution::new(Size::new(0, 0), 0.0);
    monitor.x = 0;
    monitor.y = 0;
    monitor.scale = 0.0;
    monitor
}

fn free_spot(
    boxes: &[(i64, i64, i64, i64)],
    position: Position,
    width: i64,
    height: i64,
) -> (i64, i64) {
    use crate::core::layout::Direction;

    if boxes.is_empty() {
        return (0, 0);
    }
    let right = boxes.iter().map(|(x, _, w, _)| x + w).max().unwrap();
    let left = boxes.iter().map(|(x, _, _, _)| *x).min().unwrap();
    let bottom = boxes.iter().map(|(_, y, _, h)| y + h).max().unwrap();
    let top = boxes.iter().map(|(_, y, _, _)| *y).min().unwrap();

    match position {
        Position::Toward(Direction::Left) => (left - width, 0),
        Position::Toward(Direction::Above) => (0, top - height),
        Position::Toward(Direction::Below) => (0, bottom),
        _ => (right, 0),
    }
}

impl CompositorCtl for FakeCompositor {
    fn monitors(&self) -> Result<Vec<Monitor>> {
        Ok(self.monitors.borrow().clone())
    }

    fn reload(&self) -> Result<()> {
        self.reloads.set(self.reloads.get() + 1);
        if let (true, Some(layout)) = (self.obeys.get(), self.pending.borrow().as_ref()) {
            self.apply(layout);
        }
        Ok(())
    }

    fn workspaces(&self) -> Result<Vec<Workspace>> {
        Ok(self.workspaces.borrow().clone())
    }

    fn move_workspace_to_monitor(&self, workspace: &str, monitor: &str) -> Result<()> {
        self.moved
            .borrow_mut()
            .push((workspace.into(), monitor.into()));
        Ok(())
    }

    fn render_rules(&self, layout: &Layout, _monitors: &[Monitor]) -> String {
        *self.pending.borrow_mut() = Some(layout.clone());
        format!("{} screens", layout.screens.len())
    }
}

#[derive(Default)]
struct MemoryStore {
    profiles: RefCell<HashMap<String, Layout>>,
}

fn key(monitors: &[Monitor]) -> String {
    let mut descriptions: Vec<&str> = monitors.iter().map(|m| m.description.as_str()).collect();
    descriptions.sort_unstable();
    descriptions.join("\n")
}

impl LayoutStore for MemoryStore {
    fn load(&self, monitors: &[Monitor]) -> Result<Option<Layout>> {
        Ok(self.profiles.borrow().get(&key(monitors)).cloned())
    }

    fn save(&self, monitors: &[Monitor], layout: &Layout) -> Result<()> {
        self.profiles
            .borrow_mut()
            .insert(key(monitors), layout.clone());
        Ok(())
    }

    fn render(&self, _rules: &str) -> Result<()> {
        Ok(())
    }

    fn reset(&self, monitors: &[Monitor]) -> Result<()> {
        self.profiles.borrow_mut().remove(&key(monitors));
        Ok(())
    }

    fn locked(&self, f: &mut dyn FnMut() -> Result<()>) -> Result<()> {
        f()
    }
}

fn display<'a>(compositor: &'a FakeCompositor, store: &'a MemoryStore) -> Display<'a> {
    Display {
        compositor,
        store,
        notifier: &Silent,
        settle: Settle {
            tries: 3,
            interval: Duration::ZERO,
        },
    }
}

#[test]
fn auto_extends_a_new_screen_and_then_leaves_a_matching_layout_alone() {
    let compositor = FakeCompositor::new(vec![laptop(), ultrawide()]);
    let store = MemoryStore::default();
    let display = display(&compositor, &store);

    display.auto().unwrap();

    assert_eq!(compositor.at("eDP-1"), (0, 0));
    assert_eq!(compositor.at("DP-3"), (1920, 0), "extended to the right");

    let settled = compositor.reloads.get();
    display.auto().unwrap();
    assert_eq!(
        compositor.reloads.get(),
        settled,
        "our own writes fire the hotplug hooks, so a matching layout must not reload again"
    );
}

#[test]
fn a_profile_is_restored_when_the_screen_comes_back() {
    let compositor = FakeCompositor::new(vec![laptop(), ultrawide()]);
    let store = MemoryStore::default();
    let display = display(&compositor, &store);

    display.auto().unwrap();
    display.place("eDP-1", Side::RightOf, "DP-3").unwrap();
    assert_eq!(compositor.at("DP-3"), (0, 0));
    assert_eq!(compositor.at("eDP-1"), (3440, 180));

    compositor.replug(vec![laptop()]);
    display.auto().unwrap();

    compositor.replug(vec![laptop(), ultrawide()]);
    display.auto().unwrap();

    assert_eq!(compositor.at("DP-3"), (0, 0));
    assert_eq!(compositor.at("eDP-1"), (3440, 180));
}

#[test]
fn only_moves_the_workspaces_off_a_screen_before_switching_it_off() {
    let compositor = FakeCompositor::new(vec![laptop(), ultrawide()]).with_workspaces(&[
        ("1", "eDP-1"),
        ("2", "DP-3"),
        ("special:magic", "eDP-1"),
    ]);
    let store = MemoryStore::default();
    let display = display(&compositor, &store);

    display.only("DP-3").unwrap();

    assert_eq!(
        *compositor.moved.borrow(),
        vec![("1".to_string(), "DP-3".to_string())],
        "a special workspace has no place to go"
    );
    assert!(compositor.is_disabled("eDP-1"));
    assert_eq!(compositor.at("DP-3"), (0, 0));
}

#[test]
fn a_bare_size_takes_the_highest_refresh_the_screen_offers() {
    let mut dual_refresh = laptop();
    dual_refresh.resolutions = vec![
        "1920x1080@59.951".parse().unwrap(),
        "1920x1080@60.003".parse().unwrap(),
    ];
    let compositor = FakeCompositor::new(vec![dual_refresh, ultrawide()]);
    let store = MemoryStore::default();
    let display = display(&compositor, &store);

    display.set_mode("eDP-1", "1920x1080").unwrap();

    assert_eq!(
        compositor.mode_of("eDP-1"),
        "1920x1080@60.003".parse().unwrap()
    );
}

#[test]
fn a_compositor_that_ignores_the_rules_is_reported() {
    let compositor = FakeCompositor::new(vec![laptop(), ultrawide()]);
    let store = MemoryStore::default();
    let display = display(&compositor, &store);
    compositor.obeys.set(false);

    let err = display.disable("eDP-1").unwrap_err();

    assert!(
        matches!(
            err.downcast_ref::<UserError>(),
            Some(UserError::DidNotSwitchOff { name }) if name == "eDP-1"
        ),
        "{err}"
    );
}

/// The i18n contract: anything a person can trigger from the command line has to arrive as a
/// `UserError`, because that is the only thing a catalogue can translate.
#[test]
fn every_display_failure_a_person_can_trigger_carries_a_user_error() {
    let compositor = FakeCompositor::new(vec![laptop(), ultrawide()]);
    let store = MemoryStore::default();
    let display = display(&compositor, &store);

    let failures = [
        ("unknown monitor", display.enable("DP-9").unwrap_err()),
        (
            "a size the screen has not got",
            display.set_mode("eDP-1", "800x600").unwrap_err(),
        ),
        (
            "a mode that is not a mode",
            display.set_mode("eDP-1", "enormous").unwrap_err(),
        ),
        (
            "a scale of zero",
            display.set_scale("eDP-1", "0").unwrap_err(),
        ),
        (
            "a scale that is not a number",
            display.set_scale("eDP-1", "big").unwrap_err(),
        ),
        (
            "placed against itself",
            display.place("eDP-1", Side::RightOf, "eDP-1").unwrap_err(),
        ),
        (
            "placed against an unknown screen",
            display.place("eDP-1", Side::RightOf, "DP-9").unwrap_err(),
        ),
    ];

    for (what, err) in failures {
        assert!(
            err.downcast_ref::<UserError>().is_some(),
            "{what} reaches the user untranslatable: {err:?}"
        );
    }
}
