//! The settings menu, drawn by the program: native tray menus always close after a
//! click, and this one must not. Clicking a setting applies it and leaves the menu open,
//! so several can be changed in one go; a click outside or Esc dismisses it. Groups open
//! in place, one per level. The window fits its rows, and the wheel scrolls whatever does
//! not fit on the screen.
use super::dialogs::Outcome;
use super::gfx::{self, Canvas, Font, Mouse};
use super::tray::Node;

/// The items that lead elsewhere, so the menu closes for them.
pub fn closes_menu(id: &str) -> bool {
    matches!(
        id,
        "brightness" | "posh" | "brains" | "check-update" | "about" | "exit" | "dress-custom"
    )
}

/// A group's lasting name: "Appear after: 2 seconds" stays open as its value changes.
fn group_key(label: &str) -> &str {
    label.split(':').next().unwrap_or(label)
}

pub struct MenuPanel {
    pub nodes: Vec<Node>,
    open: Vec<String>,
    scroll: i32,
    /// The height the rows want, as of the last paint.
    pub wanted_height: i32,
}

struct Row<'a> {
    node: &'a Node,
    depth: usize,
    enabled: bool,
}

fn flatten<'a>(
    nodes: &'a [Node],
    depth: usize,
    parent_on: bool,
    open: &[String],
    out: &mut Vec<Row<'a>>,
) {
    for node in nodes {
        let enabled = parent_on
            && match node {
                Node::Item { enabled, .. }
                | Node::Check { enabled, .. }
                | Node::Sub { enabled, .. } => *enabled,
                Node::Sep => true,
            };
        out.push(Row {
            node,
            depth,
            enabled,
        });
        if let Node::Sub {
            label, children, ..
        } = node
        {
            if open.get(depth).is_some_and(|k| k == group_key(label)) {
                flatten(children, depth + 1, enabled, open, out);
            }
        }
    }
}

impl MenuPanel {
    pub fn new(nodes: Vec<Node>) -> Self {
        MenuPanel {
            nodes,
            open: Vec::new(),
            scroll: 0,
            wanted_height: 0,
        }
    }

    /// How tall the window must be to show every open row.
    pub fn content_height(&self, s: f64) -> i32 {
        let (row_h, sep_h, pad) = ((26.0 * s) as i32, (9.0 * s) as i32, (6.0 * s) as i32);
        let mut rows = Vec::new();
        flatten(&self.nodes, 0, true, &self.open, &mut rows);
        let total: i32 = rows
            .iter()
            .map(|r| {
                if matches!(r.node, Node::Sep) {
                    sep_h
                } else {
                    row_h
                }
            })
            .sum();
        total + 2 * pad
    }

    pub fn draw(&mut self, c: &mut Canvas, f: &mut Font, m: &Mouse, s: f64) -> Outcome {
        c.clear(gfx::BACK);
        let px = |v: f64| (v * s) as i32;
        let text = (13.0 * s) as f32;
        let (row_h, sep_h, pad, indent) = (px(26.0), px(9.0), px(6.0), px(14.0));
        let mut rows = Vec::new();
        flatten(&self.nodes, 0, true, &self.open, &mut rows);

        let total: i32 = rows
            .iter()
            .map(|r| {
                if matches!(r.node, Node::Sep) {
                    sep_h
                } else {
                    row_h
                }
            })
            .sum();
        let view = c.h - 2 * pad;
        let max_scroll = (total - view).max(0);
        self.scroll = (self.scroll - (m.wheel * row_h as f32 * 2.0) as i32).clamp(0, max_scroll);

        let mut clicked_group: Option<(usize, String)> = None;
        let mut outcome = Outcome::None;
        let mut y = pad - self.scroll;
        for r in &rows {
            let x = pad + indent * r.depth as i32;
            if let Node::Sep = r.node {
                if y + sep_h > pad && y < c.h - pad {
                    c.rect(
                        x + px(6.0),
                        y + sep_h / 2,
                        c.w - x - pad - px(6.0),
                        1,
                        gfx::RIM,
                    );
                }
                y += sep_h;
                continue;
            }
            if y + row_h > pad && y < c.h - pad {
                let hover = r.enabled
                    && m.inside(pad, y, c.w - 2 * pad, row_h)
                    && m.y >= pad
                    && m.y < c.h - pad;
                if hover {
                    c.round_box(pad, y, c.w - 2 * pad, row_h, 4.0 * s, gfx::RIM, 0, 0.0);
                }
                let color = if r.enabled { gfx::WHITE } else { gfx::DIM };
                let text_y = y + px(5.0);
                let text_x = x + px(26.0);
                let max = c.w - text_x - pad - px(20.0);
                match r.node {
                    Node::Item { id, label, .. } => {
                        c.text_fit(f, text_x, text_y, max, label, text, color);
                        if hover && m.clicked {
                            outcome = Outcome::Menu(id.clone());
                        }
                    }
                    Node::Check {
                        id, label, checked, ..
                    } => {
                        let (bx, by, b) = (x + px(6.0), y + px(6.0), px(14.0));
                        let fill = if *checked {
                            if r.enabled {
                                gfx::ACCENT
                            } else {
                                gfx::DIM
                            }
                        } else {
                            gfx::PANEL
                        };
                        c.round_box(bx, by, b, b, 3.0 * s, fill, gfx::DIM, 1.0);
                        c.text_fit(f, text_x, text_y, max, label, text, color);
                        if hover && m.clicked {
                            outcome = Outcome::Menu(id.clone());
                        }
                    }
                    Node::Sub { label, .. } => {
                        let key = group_key(label);
                        let is_open = self.open.get(r.depth).is_some_and(|k| k == key);
                        c.text_fit(f, text_x, text_y, max, label, text, color);
                        c.text_right(
                            f,
                            c.w - pad - px(8.0),
                            text_y,
                            if is_open { "-" } else { "+" },
                            text,
                            gfx::DIM,
                        );
                        if hover && m.clicked {
                            clicked_group = Some((r.depth, key.to_string()));
                        }
                    }
                    Node::Sep => {}
                }
            }
            y += row_h;
        }
        if let Some((depth, key)) = clicked_group {
            let was_open = self.open.get(depth).is_some_and(|k| *k == key);
            self.open.truncate(depth);
            if !was_open {
                self.open.push(key);
            }
        }
        self.wanted_height = self.content_height(s);
        // Padding above and below hides rows scrolled part-way out; then the rim and thumb.
        c.rect(0, 0, c.w, pad, gfx::BACK);
        c.rect(0, c.h - pad, c.w, pad, gfx::BACK);
        if max_scroll > 0 {
            let thumb = (view * view / total).max(px(20.0));
            let ty = pad + (view - thumb) * self.scroll / max_scroll;
            c.round_box(c.w - px(5.0), ty, px(3.0), thumb, 1.5 * s, gfx::DIM, 0, 0.0);
        }
        c.round_box(0, 0, c.w, c.h, 1.0, 0x00000000, gfx::RIM, 1.0);
        outcome
    }
}
