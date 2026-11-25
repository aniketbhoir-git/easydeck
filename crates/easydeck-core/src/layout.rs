//! Layout rules (SPEC §4.2). Explicit positions are kept. A widget without a position goes
//! in the first free cell in reading order (row by row, left to right) and covers one cell.
//! Two widgets must not overlap and each must fit in the grid.

use crate::deck::{Deck, Page, Position};
use std::fmt;

/// A widget with its final cell range on the grid. `index` is the widget's index in
/// `page.widgets`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Placed {
    pub index: usize,
    pub id: String,
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

/// A layout problem. `index` fields are indexes in `page.widgets`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutError {
    /// An explicit position reaches outside the grid.
    OffGrid {
        index: usize,
        id: String,
        right: u64,
        bottom: u64,
    },
    /// Two explicit positions share a cell. `index` is the later widget.
    Overlap {
        index: usize,
        id: String,
        other_id: String,
    },
    /// A widget without a position found no free cell.
    NoSpace { index: usize, id: String },
}

impl LayoutError {
    /// Index in `page.widgets` of the widget the error is reported on.
    pub fn index(&self) -> usize {
        match self {
            Self::OffGrid { index, .. }
            | Self::Overlap { index, .. }
            | Self::NoSpace { index, .. } => *index,
        }
    }
}

impl fmt::Display for LayoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OffGrid {
                id, right, bottom, ..
            } => write!(f, "widget \"{id}\" spans to column {right}, row {bottom}"),
            Self::Overlap { id, other_id, .. } => {
                write!(f, "widgets \"{other_id}\" and \"{id}\" overlap")
            }
            Self::NoSpace { id, .. } => {
                write!(
                    f,
                    "widget \"{id}\" has no position and no free cell is left"
                )
            }
        }
    }
}

/// Place every widget of a page. Returns the placement in widget order, or every problem found.
/// Expects a page that passed schema validation (grid of 1 to 24 columns and rows).
pub fn place(page: &Page) -> Result<Vec<Placed>, Vec<LayoutError>> {
    let (cols, rows) = (page.grid.columns as usize, page.grid.rows as usize);
    // owner[row * cols + col] is the widget index that covers the cell.
    let mut owner: Vec<Option<usize>> = vec![None; cols * rows];
    let mut errors = Vec::new();
    let mut placed: Vec<Option<Placed>> = vec![None; page.widgets.len()];

    for (index, widget) in page.widgets.iter().enumerate() {
        let Some(p) = widget.position else { continue };
        let (right, bottom) = (
            u64::from(p.x) + u64::from(p.w),
            u64::from(p.y) + u64::from(p.h),
        );
        if right > cols as u64 || bottom > rows as u64 {
            errors.push(LayoutError::OffGrid {
                index,
                id: widget.id.clone(),
                right,
                bottom,
            });
            continue;
        }
        let mut hit: Vec<usize> = Vec::new();
        for row in p.y as usize..bottom as usize {
            for col in p.x as usize..right as usize {
                match owner[row * cols + col] {
                    Some(o) if !hit.contains(&o) => hit.push(o),
                    Some(_) => {}
                    None => owner[row * cols + col] = Some(index),
                }
            }
        }
        for o in hit {
            errors.push(LayoutError::Overlap {
                index,
                id: widget.id.clone(),
                other_id: page.widgets[o].id.clone(),
            });
        }
        placed[index] = Some(Placed {
            index,
            id: widget.id.clone(),
            x: p.x,
            y: p.y,
            w: p.w,
            h: p.h,
        });
    }

    for (index, widget) in page.widgets.iter().enumerate() {
        if widget.position.is_some() {
            continue;
        }
        let free = (0..rows)
            .flat_map(|row| (0..cols).map(move |col| (row, col)))
            .find(|&(row, col)| owner[row * cols + col].is_none());
        match free {
            Some((row, col)) => {
                owner[row * cols + col] = Some(index);
                placed[index] = Some(Placed {
                    index,
                    id: widget.id.clone(),
                    x: col as u32,
                    y: row as u32,
                    w: 1,
                    h: 1,
                });
            }
            None => errors.push(LayoutError::NoSpace {
                index,
                id: widget.id.clone(),
            }),
        }
    }

    if errors.is_empty() {
        Ok(placed.into_iter().flatten().collect())
    } else {
        Err(errors)
    }
}

/// A copy of the deck with every widget position filled in. This is what the UI draws.
/// A page that cannot be placed keeps its widgets as they are; validation reports it.
/// Never save the result: it fixes the automatic positions. Save the original deck.
pub fn resolve(deck: &Deck) -> Deck {
    let mut out = deck.clone();
    for page in &mut out.pages {
        if let Ok(placed) = place(page) {
            for (widget, p) in page.widgets.iter_mut().zip(placed) {
                widget.position = Some(Position {
                    x: p.x,
                    y: p.y,
                    w: p.w,
                    h: p.h,
                });
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deck::{Grid, Widget, WidgetKind};
    use proptest::prelude::*;

    fn widget(id: usize, position: Option<Position>) -> Widget {
        Widget {
            id: format!("w{id}"),
            label: None,
            icon: None,
            position,
            kind: WidgetKind::Text {
                kind: None,
                text: None,
                hour12: None,
                seconds: None,
                show_date: None,
            },
        }
    }

    fn page(columns: u32, rows: u32, positions: Vec<Option<Position>>) -> Page {
        Page {
            id: "p".into(),
            title: "P".into(),
            grid: Grid { columns, rows },
            trigger: None,
            widgets: positions
                .into_iter()
                .enumerate()
                .map(|(i, p)| widget(i, p))
                .collect(),
        }
    }

    fn pos(x: u32, y: u32, w: u32, h: u32) -> Option<Position> {
        Some(Position { x, y, w, h })
    }

    fn cells(p: &Placed) -> Vec<(u32, u32)> {
        (p.y..p.y + p.h)
            .flat_map(|y| (p.x..p.x + p.w).map(move |x| (x, y)))
            .collect()
    }

    #[test]
    fn unpositioned_widgets_fill_in_reading_order() {
        let placed = place(&page(3, 2, vec![None; 4])).unwrap();
        let at: Vec<_> = placed.iter().map(|p| (p.x, p.y)).collect();
        assert_eq!(at, [(0, 0), (1, 0), (2, 0), (0, 1)]);
    }

    #[test]
    fn explicit_positions_are_kept_and_skipped_by_auto_placement() {
        let placed = place(&page(3, 2, vec![None, pos(0, 0, 2, 1), None])).unwrap();
        assert_eq!((placed[0].x, placed[0].y), (2, 0));
        assert_eq!((placed[1].x, placed[1].y, placed[1].w), (0, 0, 2));
        assert_eq!((placed[2].x, placed[2].y), (0, 1));
    }

    #[test]
    fn a_widget_that_fits_nowhere_is_an_error() {
        let errors = place(&page(2, 1, vec![None; 3])).unwrap_err();
        assert_eq!(
            errors,
            [LayoutError::NoSpace {
                index: 2,
                id: "w2".into()
            }]
        );
    }

    #[test]
    fn overlap_names_both_widgets() {
        let errors = place(&page(4, 2, vec![pos(0, 0, 2, 2), pos(1, 1, 2, 1)])).unwrap_err();
        assert_eq!(
            errors,
            [LayoutError::Overlap {
                index: 1,
                id: "w1".into(),
                other_id: "w0".into()
            }]
        );
    }

    #[test]
    fn out_of_grid_is_an_error() {
        let errors = place(&page(2, 2, vec![pos(1, 0, 2, 1)])).unwrap_err();
        assert!(matches!(errors[0], LayoutError::OffGrid { index: 0, .. }));
    }

    #[test]
    fn resolve_fills_positions() {
        let deck = Deck {
            schema: None,
            version: 1,
            name: "d".into(),
            settings: None,
            sources: Default::default(),
            pages: vec![page(2, 1, vec![None, None])],
        };
        let out = resolve(&deck);
        assert_eq!(out.pages[0].widgets[1].position, pos(1, 0, 1, 1));
    }

    /// A grid size and positions for it. Most positions fall inside the grid; some reach
    /// outside it, and some are missing.
    fn arb_page() -> impl Strategy<Value = (u32, u32, Vec<Option<Position>>)> {
        (1u32..=8, 1u32..=8).prop_flat_map(|(columns, rows)| {
            let inside = (0..columns, 0..rows).prop_flat_map(move |(x, y)| {
                (1..=columns - x, 1..=rows - y).prop_map(move |(w, h)| pos(x, y, w, h))
            });
            let anywhere =
                (0u32..10, 0u32..10, 1u32..4, 1u32..4).prop_map(|(x, y, w, h)| pos(x, y, w, h));
            let one = prop_oneof![
                3 => inside,
                1 => anywhere,
                2 => Just(None),
            ];
            (
                Just(columns),
                Just(rows),
                proptest::collection::vec(one, 0..20),
            )
        })
    }

    /// Independent check: is there a real problem in this input? True when an explicit
    /// widget leaves the grid, two explicit widgets share a cell, or the widgets without a
    /// position outnumber the cells left free.
    fn has_real_problem(columns: u32, rows: u32, positions: &[Option<Position>]) -> bool {
        let explicit: Vec<Position> = positions.iter().flatten().copied().collect();
        if explicit
            .iter()
            .any(|p| p.x + p.w > columns || p.y + p.h > rows)
        {
            return true;
        }
        for (i, a) in explicit.iter().enumerate() {
            for b in &explicit[i + 1..] {
                if a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h {
                    return true;
                }
            }
        }
        let used: u32 = explicit.iter().map(|p| p.w * p.h).sum();
        let auto = positions.iter().filter(|p| p.is_none()).count() as u32;
        auto > columns * rows - used
    }

    proptest! {
        /// Whatever the input, a successful placement has no overlap and stays in the grid.
        #[test]
        fn placed_widgets_never_overlap_and_fit((columns, rows, positions) in arb_page()) {
            let n = positions.len();
            if let Ok(placed) = place(&page(columns, rows, positions)) {
                prop_assert_eq!(placed.len(), n);
                let mut seen = std::collections::HashSet::new();
                for p in &placed {
                    prop_assert!(p.x + p.w <= columns && p.y + p.h <= rows);
                    for c in cells(p) {
                        prop_assert!(seen.insert(c), "cell {:?} used twice", c);
                    }
                }
            }
        }

        /// On success, every explicit position comes back unchanged.
        #[test]
        fn explicit_positions_come_back_unchanged((columns, rows, positions) in arb_page()) {
            if let Ok(placed) = place(&page(columns, rows, positions.clone())) {
                for (p, want) in placed.iter().zip(&positions) {
                    if let Some(want) = want {
                        prop_assert_eq!((p.x, p.y, p.w, p.h), (want.x, want.y, want.w, want.h));
                    }
                }
            }
        }

        /// On failure, an independent check finds a real overlap, off-grid widget or full grid.
        #[test]
        fn failure_means_a_real_problem((columns, rows, positions) in arb_page()) {
            if place(&page(columns, rows, positions.clone())).is_err() {
                prop_assert!(has_real_problem(columns, rows, &positions));
            }
        }

        /// A page with a real problem never places.
        #[test]
        fn a_real_problem_always_fails((columns, rows, positions) in arb_page()) {
            if has_real_problem(columns, rows, &positions) {
                prop_assert!(place(&page(columns, rows, positions)).is_err());
            }
        }

        /// Widgets without positions always fit while free cells remain.
        #[test]
        fn auto_widgets_fit_when_cells_remain(columns in 1u32..=8, rows in 1u32..=8, n in 0u32..64) {
            let n = n.min(columns * rows) as usize;
            prop_assert!(place(&page(columns, rows, vec![None; n])).is_ok());
        }
    }
}
