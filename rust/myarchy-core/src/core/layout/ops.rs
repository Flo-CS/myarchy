use std::collections::BTreeMap;

use anyhow::{Result, bail};

use super::{Axis, Direction, Layout, Mode, Placement, Point, Position, Scale, Side, State};
use crate::core::error::UserError;
use crate::core::resolution::Size;

impl Layout {
    /// Every screen on, every mode back to its preferred one, laid out along `direction` from the
    /// anchor. The reliable way back from any layout.
    pub fn extend(&mut self, anchor_desc: &str, direction: Direction) {
        for (desc, screen) in &mut self.screens {
            screen.state = State::On;
            screen.placement = Placement {
                mode: Mode::Preferred,
                position: if desc == anchor_desc {
                    Position::At(Point { x: 0, y: 0 })
                } else {
                    Position::Toward(direction)
                },
                scale: screen.placement.scale,
            };
        }
    }

    pub fn mirror(&mut self, anchor_desc: &str) {
        for (desc, screen) in &mut self.screens {
            screen.state = if desc == anchor_desc {
                State::On
            } else {
                State::Mirroring(anchor_desc.to_string())
            };
        }
        if let Some(screen) = self.screens.get_mut(anchor_desc) {
            screen.placement.position = Position::At(Point { x: 0, y: 0 });
        }
    }

    pub fn only(&mut self, keep_desc: &str) {
        for (desc, screen) in &mut self.screens {
            if desc == keep_desc {
                screen.state = State::On;
                screen.placement.position = Position::At(Point { x: 0, y: 0 });
            } else {
                screen.state = State::Off;
            }
        }
        self.anchor = Some(keep_desc.to_string());
    }

    /// Lays the whole axis out edge to edge, which is what makes an overlap impossible, and
    /// centers every screen on the other axis so the shared border is always the full `min(size)`.
    pub fn place(&mut self, moving_desc: &str, side: Side, reference_desc: &str) -> Result<()> {
        if moving_desc == reference_desc {
            bail!(UserError::CannotPlaceSelfRelative);
        }

        let axis = side.axis();
        let sizes = self.logical_sizes();
        if !sizes.contains_key(moving_desc) || !sizes.contains_key(reference_desc) {
            bail!(UserError::CannotPlace {
                name: moving_desc.to_string(),
                side,
                reference: reference_desc.to_string(),
            });
        }

        let order = self.order_along(axis, moving_desc, side, reference_desc);
        for (desc, point) in pack(axis, &order, &sizes) {
            self.require(&desc)?.placement.position = Position::At(point);
        }
        Ok(())
    }

    pub fn set_mode(&mut self, desc: &str, mode: Mode) -> Result<()> {
        self.require(desc)?.placement.mode = mode;
        Ok(())
    }

    pub fn set_scale(&mut self, desc: &str, scale: Scale) -> Result<()> {
        self.require(desc)?.placement.scale = scale;
        Ok(())
    }

    /// Also the way out of mirroring, since a mirror is on but has no place on the layout.
    pub fn enable(&mut self, desc: &str) -> Result<()> {
        let screen = self.require(desc)?;
        if !screen.is_on() {
            if screen.placement.logical_size().is_none() {
                screen.placement.position = Position::Auto;
            }
            screen.state = State::On;
        }
        Ok(())
    }

    pub fn disable(&mut self, desc: &str) -> Result<()> {
        if self.enabled_count() <= 1 {
            bail!(UserError::CannotDisableLastScreen);
        }
        self.require(desc)?.state = State::Off;
        Ok(())
    }

    /// A screen only has a resolved size once the compositor has told us one, so an unapplied
    /// `preferred`/`auto` screen cannot take part in the axis.
    fn logical_sizes(&self) -> BTreeMap<String, Size> {
        self.on()
            .filter_map(|(desc, screen)| Some((desc.clone(), screen.placement.logical_size()?)))
            .collect()
    }

    fn order_along(
        &self,
        axis: Axis,
        moving_desc: &str,
        side: Side,
        reference_desc: &str,
    ) -> Vec<String> {
        let mut rest: Vec<(&String, i64)> = self
            .on()
            .filter(|(desc, _)| desc.as_str() != moving_desc)
            .filter_map(|(desc, screen)| match screen.placement.position {
                Position::At(p) => Some((desc, if axis == Axis::Horizontal { p.x } else { p.y })),
                _ => None,
            })
            .collect();
        rest.sort_by_key(|(_, coord)| *coord);

        let mut order = Vec::with_capacity(rest.len() + 1);
        for (desc, _) in rest {
            if desc == reference_desc && side.before_reference() {
                order.push(moving_desc.to_string());
            }
            order.push(desc.clone());
            if desc == reference_desc && !side.before_reference() {
                order.push(moving_desc.to_string());
            }
        }
        order
    }
}

fn pack(axis: Axis, order: &[String], sizes: &BTreeMap<String, Size>) -> Vec<(String, Point)> {
    let extent = order
        .iter()
        .filter_map(|desc| sizes.get(desc))
        .map(|size| match axis {
            Axis::Horizontal => size.height,
            Axis::Vertical => size.width,
        })
        .max()
        .unwrap_or(0);

    let mut offset = 0;
    let mut placed = Vec::with_capacity(order.len());
    for desc in order {
        let Some(size) = sizes.get(desc) else {
            continue;
        };
        let (along, across) = match axis {
            Axis::Horizontal => (size.width, size.height),
            Axis::Vertical => (size.height, size.width),
        };
        let centered = (extent - across) / 2;
        placed.push((
            desc.clone(),
            match axis {
                Axis::Horizontal => Point {
                    x: offset,
                    y: centered,
                },
                Axis::Vertical => Point {
                    x: centered,
                    y: offset,
                },
            },
        ));
        offset += along;
    }
    placed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::layout::Screen;
    use crate::core::layout::fixtures::{BOE, DELL, LG};

    fn screen(state: State, mode: &str, position: &str, scale: &str) -> Screen {
        Screen {
            state,
            placement: Placement {
                mode: mode.parse().unwrap(),
                position: position.parse().unwrap(),
                scale: scale.parse().unwrap(),
            },
        }
    }

    fn on(mode: &str, position: &str) -> Screen {
        screen(State::On, mode, position, "1")
    }

    fn layout_of(entries: &[(&str, Screen)]) -> Layout {
        let mut layout = Layout::default();
        for (desc, screen) in entries {
            layout.screens.insert((*desc).into(), screen.clone());
        }
        layout
    }

    fn pair() -> Layout {
        layout_of(&[
            (BOE, on("1920x1080@60.003", "0x0")),
            (LG, on("3440x1440@99.997", "5000x900")),
        ])
    }

    fn trio() -> Layout {
        layout_of(&[
            (BOE, on("1920x1080@60.003", "0x0")),
            (LG, on("3440x1440@99.997", "1920x0")),
            (DELL, on("1920x1200@59.95", "5360x0")),
        ])
    }

    fn position_of(layout: &Layout, desc: &str) -> Position {
        layout.screens[desc].placement.position
    }

    fn at(s: &str) -> Position {
        s.parse().unwrap()
    }

    fn rects(layout: &Layout) -> Vec<(&str, i64, i64, i64, i64)> {
        layout
            .on()
            .filter_map(|(desc, s)| {
                let Position::At(p) = s.placement.position else {
                    return None;
                };
                let size = s.placement.logical_size()?;
                Some((desc.as_str(), p.x, p.y, size.width, size.height))
            })
            .collect()
    }

    #[track_caller]
    fn assert_laid_out_along(layout: &Layout, axis: Axis) {
        let rects = rects(layout);
        for (i, a) in rects.iter().enumerate() {
            for b in &rects[i + 1..] {
                let apart =
                    a.1 + a.3 <= b.1 || b.1 + b.3 <= a.1 || a.2 + a.4 <= b.2 || b.2 + b.4 <= a.2;
                assert!(apart, "{} and {} overlap: {a:?} {b:?}", a.0, b.0);
            }
        }

        let mut spans: Vec<(i64, i64)> = rects
            .iter()
            .map(|(_, x, y, w, h)| match axis {
                Axis::Horizontal => (*x, *w),
                Axis::Vertical => (*y, *h),
            })
            .collect();
        spans.sort();
        assert_eq!(spans[0].0, 0, "the row does not start at the origin");
        for window in spans.windows(2) {
            let (start, size) = window[0];
            assert_eq!(start + size, window[1].0, "gap or overlap along the axis");
        }
    }

    #[test]
    fn place_lays_the_row_edge_to_edge_and_centers_it() {
        let mut layout = pair();
        layout.place(LG, Side::RightOf, BOE).unwrap();

        assert_eq!(position_of(&layout, BOE), at("0x180"));
        assert_eq!(position_of(&layout, LG), at("1920x0"));
        assert_laid_out_along(&layout, Axis::Horizontal);
    }

    #[test]
    fn place_reorders_the_row_around_the_reference() {
        let mut left = pair();
        left.place(LG, Side::LeftOf, BOE).unwrap();
        assert_eq!(position_of(&left, LG), at("0x0"));
        assert_eq!(position_of(&left, BOE), at("3440x180"));

        let mut middle = trio();
        middle.place(DELL, Side::RightOf, BOE).unwrap();
        assert_eq!(position_of(&middle, BOE), at("0x180"));
        assert_eq!(position_of(&middle, DELL), at("1920x120"));
        assert_eq!(position_of(&middle, LG), at("3840x0"));
        assert_laid_out_along(&middle, Axis::Horizontal);
    }

    #[test]
    fn place_stacks_on_the_vertical_axis_centered_on_x() {
        let mut below = pair();
        below.place(BOE, Side::Below, LG).unwrap();
        assert_eq!(position_of(&below, LG), at("0x0"));
        assert_eq!(position_of(&below, BOE), at("760x1440"));

        let mut above = pair();
        above.place(BOE, Side::Above, LG).unwrap();
        assert_eq!(position_of(&above, BOE), at("760x0"));
        assert_eq!(position_of(&above, LG), at("0x1080"));
        assert_laid_out_along(&above, Axis::Vertical);
    }

    #[test]
    fn place_never_overlaps_whatever_the_order() {
        for side in [Side::LeftOf, Side::RightOf, Side::Above, Side::Below] {
            for (moving, reference) in [(BOE, LG), (LG, DELL), (DELL, BOE), (LG, BOE)] {
                let mut layout = trio();
                layout.place(moving, side, reference).unwrap();
                assert_laid_out_along(&layout, side.axis());
            }
        }
    }

    #[test]
    fn place_is_idempotent() {
        let mut layout = trio();
        layout.place(LG, Side::RightOf, BOE).unwrap();
        let once = layout.clone();
        layout.place(LG, Side::RightOf, BOE).unwrap();

        assert_eq!(layout, once);
    }

    #[test]
    fn place_measures_in_logical_pixels() {
        let mut layout = pair();
        layout.screens.get_mut(BOE).unwrap().placement.scale = Scale::factor(2.0);
        layout.place(LG, Side::RightOf, BOE).unwrap();

        assert_eq!(position_of(&layout, BOE), at("0x450"));
        assert_eq!(position_of(&layout, LG), at("960x0"));
    }

    #[test]
    fn place_leaves_screens_that_are_not_in_the_row_where_they_were() {
        let mut layout = trio();
        layout.screens.get_mut(DELL).unwrap().state = State::Off;
        layout.place(LG, Side::RightOf, BOE).unwrap();
        assert_eq!(position_of(&layout, DELL), at("5360x0"));

        let mut mirrored = trio();
        mirrored.screens.get_mut(DELL).unwrap().state = State::Mirroring(BOE.into());
        mirrored.place(LG, Side::RightOf, BOE).unwrap();
        assert_eq!(position_of(&mirrored, DELL), at("5360x0"));
        assert_eq!(position_of(&mirrored, LG), at("1920x0"));
    }

    #[test]
    fn place_refuses_what_it_cannot_lay_out() {
        let mut layout = pair();
        assert!(layout.place(LG, Side::RightOf, LG).is_err(), "itself");
        assert!(layout.place("ghost", Side::RightOf, BOE).is_err());
        assert!(layout.place(LG, Side::RightOf, "ghost").is_err());

        let mut dark = pair();
        dark.screens.get_mut(BOE).unwrap().state = State::Off;
        assert!(dark.place(LG, Side::RightOf, BOE).is_err());

        let mut unsized_yet = pair();
        unsized_yet.screens.get_mut(BOE).unwrap().placement.mode = Mode::Preferred;
        assert!(
            unsized_yet.place(LG, Side::RightOf, BOE).is_err(),
            "no size yet"
        );
    }

    #[test]
    fn extend_resets_the_whole_layout_around_the_anchor() {
        let mut layout = trio();
        layout.screens.get_mut(BOE).unwrap().state = State::Off;
        layout.screens.get_mut(DELL).unwrap().state = State::Mirroring(LG.into());
        layout.screens.get_mut(BOE).unwrap().placement.scale = Scale::factor(1.25);

        layout.extend(LG, Direction::Right);

        assert!(layout.screens.values().all(|s| s.state == State::On));
        assert!(
            layout
                .screens
                .values()
                .all(|s| s.placement.mode == Mode::Preferred)
        );
        assert_eq!(position_of(&layout, LG), at("0x0"));
        assert_eq!(
            position_of(&layout, BOE),
            Position::Toward(Direction::Right)
        );
        assert_eq!(
            position_of(&layout, DELL),
            Position::Toward(Direction::Right)
        );
        assert_eq!(
            layout.screens[BOE].placement.scale,
            Scale::factor(1.25),
            "extend resets the layout, not the scale"
        );
    }

    #[test]
    fn extend_maps_every_direction() {
        for direction in [
            Direction::Left,
            Direction::Right,
            Direction::Above,
            Direction::Below,
        ] {
            let mut layout = pair();
            layout.extend(BOE, direction);
            assert_eq!(position_of(&layout, LG), Position::Toward(direction));
        }
    }

    #[test]
    fn mirror_duplicates_every_other_screen_onto_the_anchor() {
        let mut layout = trio();
        layout.screens.get_mut(DELL).unwrap().state = State::Off;
        layout.screens.get_mut(BOE).unwrap().placement.scale = Scale::factor(1.5);

        layout.mirror(LG);

        assert_eq!(layout.screens[LG].state, State::On);
        assert_eq!(position_of(&layout, LG), at("0x0"));
        assert_eq!(layout.screens[BOE].state, State::Mirroring(LG.into()));
        assert_eq!(
            layout.screens[DELL].state,
            State::Mirroring(LG.into()),
            "was off"
        );
        assert_eq!(
            layout.screens[BOE].placement.mode,
            "1920x1080@60.003".parse().unwrap()
        );
        assert_eq!(layout.screens[BOE].placement.scale, Scale::factor(1.5));
    }

    #[test]
    fn mirror_re_anchoring_repoints_every_mirror() {
        let mut layout = trio();
        layout.mirror(LG);
        layout.mirror(BOE);

        assert_eq!(layout.screens[BOE].state, State::On);
        assert_eq!(layout.screens[LG].state, State::Mirroring(BOE.into()));
        assert_eq!(layout.screens[DELL].state, State::Mirroring(BOE.into()));
    }

    #[test]
    fn only_leaves_one_screen_on_and_anchors_it() {
        let mut layout = trio();
        layout.mirror(LG);
        layout.only(BOE);

        assert_eq!(layout.screens[BOE].state, State::On);
        assert_eq!(position_of(&layout, BOE), at("0x0"));
        assert!(layout.screens[LG].is_off());
        assert!(layout.screens[DELL].is_off());
        assert_eq!(layout.anchor.as_deref(), Some(BOE));
    }

    #[test]
    fn a_screen_switched_off_keeps_the_geometry_it_had() {
        let mut layout = pair();
        layout.disable(LG).unwrap();

        assert_eq!(position_of(&layout, LG), at("5000x900"));
        assert_eq!(
            layout.screens[LG].placement.mode,
            "3440x1440@99.997".parse().unwrap()
        );
    }

    #[test]
    fn enable_switches_a_screen_back_on_or_ends_its_mirroring() {
        let mut dark = pair();
        dark.disable(LG).unwrap();
        dark.enable(LG).unwrap();
        assert_eq!(dark.screens[LG].state, State::On);
        assert_eq!(position_of(&dark, LG), at("5000x900"), "kept its place");

        let mut mirrored = pair();
        mirrored.mirror(LG);
        mirrored.enable(BOE).unwrap();
        assert_eq!(mirrored.screens[BOE].state, State::On);

        let mut fresh = layout_of(&[("NEW", screen(State::Off, "preferred", "0x0", "auto"))]);
        fresh.enable("NEW").unwrap();
        assert_eq!(position_of(&fresh, "NEW"), Position::Auto, "no size yet");
    }

    #[test]
    fn the_last_screen_on_cannot_be_switched_off() {
        let mut layout = pair();
        layout.disable(BOE).unwrap();
        assert!(layout.disable(LG).is_err());

        let mut mirrored = pair();
        mirrored.mirror(LG);
        assert!(mirrored.disable(BOE).is_ok(), "a mirror is still lit");
        assert!(mirrored.disable(LG).is_err());
    }
}
