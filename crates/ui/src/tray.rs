//! The tray icon and its menu, grouped by feature: Walkers (the master switch plus the
//! shared "Appear after" delay), then the cast, then Hot corners, then the program.
//! The menu is described as plain data from the current choices and shown by
//! menu_panel.rs. The icon itself comes from `tray-icon` on Windows and macOS, and the
//! StatusNotifierItem D-Bus protocol on Linux. A left-click opens the brightness panel.
use super::UserEvent;
use crate::prelude::*;
pub use idlewalker_application::Choices;
use idlewalker_application::Desktop;
use winit::event_loop::EventLoopProxy;

#[derive(Clone)]
pub enum Node {
    Item {
        id: String,
        label: String,
        enabled: bool,
    },
    Check {
        id: String,
        label: String,
        enabled: bool,
        checked: bool,
    },
    Sub {
        label: String,
        enabled: bool,
        children: Vec<Node>,
    },
    Sep,
}

fn item(id: &str, label: &str) -> Node {
    Node::Item {
        id: id.into(),
        label: label.into(),
        enabled: true,
    }
}
fn check(id: impl Into<String>, label: impl Into<String>, enabled: bool, checked: bool) -> Node {
    Node::Check {
        id: id.into(),
        label: label.into(),
        enabled,
        checked,
    }
}

pub const CORNER_NAMES: [&str; 4] = ["Top-left", "Top-right", "Bottom-left", "Bottom-right"];

/// "1 second", "2 seconds", "1 minute", "5 minutes".
pub fn describe_idle(seconds: i32) -> String {
    let n = if seconds >= 60 { seconds / 60 } else { seconds };
    format!(
        "{} {}{}",
        n,
        if seconds >= 60 { "minute" } else { "second" },
        if n == 1 { "" } else { "s" }
    )
}

/// "0.3 s", "1 s".
fn describe_delay(ms: i32) -> String {
    format!("{} s", rt::format_num(ms as f64 / 1000.0, "0.#"))
}

fn is_named_dress(rgb: i32) -> Option<i32> {
    (0..SETTINGS_DRESS_COLORS.len()).find(|&i| SETTINGS_DRESS_COLORS[i] == rgb)
}

fn describe_dress(rgb: i32) -> String {
    if rgb == Settings::RANDOM_DRESS {
        return "Random".into();
    }
    match is_named_dress(rgb) {
        Some(i) => SETTINGS_DRESS_NAMES[i].clone(),
        None => format!("Custom #{:06X}", rgb),
    }
}

/// The whole menu, as the choices stand. A feature's settings are greyed while it is off;
/// group headers stay enabled so a group can always be opened.
pub fn build(c: &Choices, desktop: &dyn Desktop) -> Vec<Node> {
    let action_names = desktop.corner_action_names();
    let w = c.walkers;

    let mut idle = Vec::new();
    let mut minutes = false;
    for s in SETTINGS_IDLE_CHOICES.items() {
        if s >= 60 && !minutes {
            minutes = true;
            idle.push(Node::Sep);
        }
        idle.push(check(
            format!("idle:{}", s),
            describe_idle(s),
            true,
            s == c.idle_seconds,
        ));
    }

    let minions_on = w && c.minions;
    let mut dress: Vec<Node> = (0..SETTINGS_DRESS_COLORS.len())
        .map(|i| {
            let v = SETTINGS_DRESS_COLORS[i];
            check(
                format!("dress:{}", v),
                SETTINGS_DRESS_NAMES[i].clone(),
                true,
                v == c.dress,
            )
        })
        .collect();
    dress.push(Node::Sep);
    dress.push(check(
        format!("dress:{}", Settings::RANDOM_DRESS),
        "Random",
        true,
        c.dress == Settings::RANDOM_DRESS,
    ));
    let custom = c.dress != Settings::RANDOM_DRESS && is_named_dress(c.dress).is_none();
    dress.push(check("dress-custom", "Custom...", true, custom));

    let all_minions = (1 << Settings::MAX_WALKERS) - 1;
    let mut minions = vec![
        check("minions", "Enabled", true, c.minions),
        check(
            "minions-all",
            "All",
            minions_on,
            c.minion_mask == all_minions,
        ),
        Node::Sep,
    ];
    for i in 0..Settings::MAX_WALKERS {
        minions.push(check(
            format!("minion:{}", i),
            PERSONALITY_ROSTER[i].name.clone(),
            minions_on,
            (c.minion_mask >> i) & 1 != 0,
        ));
    }
    minions.push(Node::Sep);
    minions.push(Node::Sub {
        label: format!("Dress: {}", describe_dress(c.dress)),
        enabled: minions_on,
        children: dress,
    });

    let birds_on = w && c.birds;
    let all_birds = (1 << Settings::MAX_BIRDS) - 1;
    let mut birds = vec![
        check("birds", "Enabled", true, c.birds),
        check("birds-all", "All", birds_on, c.bird_mask == all_birds),
        Node::Sep,
    ];
    for i in 0..Settings::MAX_BIRDS {
        birds.push(check(
            format!("bird:{}", i),
            PERSONALITY_BIRD_ROSTER[i].name.clone(),
            birds_on,
            (c.bird_mask >> i) & 1 != 0,
        ));
    }

    let mut corners = vec![check("corners", "Enabled", true, c.corners), Node::Sep];
    for (i, name) in CORNER_NAMES.iter().enumerate() {
        let chosen = c.actions[i] as usize;
        let children = action_names
            .iter()
            .enumerate()
            .map(|(a, n)| check(format!("corner:{}:{}", i, a), *n, true, a == chosen))
            .collect();
        corners.push(Node::Sub {
            label: format!("{}: {}", name, action_names[chosen]),
            enabled: c.corners,
            children,
        });
    }
    corners.push(Node::Sep);
    let delays = SETTINGS_HOT_CORNER_DELAY_CHOICES
        .items()
        .map(|ms| {
            check(
                format!("delay:{}", ms),
                describe_delay(ms),
                true,
                ms == c.delay_ms,
            )
        })
        .collect();
    corners.push(Node::Sub {
        label: format!("Delay: {}", describe_delay(c.delay_ms)),
        enabled: c.corners,
        children: delays,
    });

    vec![
        check("walkers", "Walkers", true, c.walkers),
        Node::Sub {
            label: format!("Appear after: {}", describe_idle(c.idle_seconds)),
            enabled: w,
            children: idle,
        },
        check("hide-on-move", "Hide on mouse move", w, c.hide_on_move),
        Node::Sep,
        Node::Sub {
            label: "Minions".into(),
            enabled: w,
            children: minions,
        },
        Node::Sub {
            label: "Angry Birds".into(),
            enabled: w,
            children: birds,
        },
        Node::Sep,
        Node::Sub {
            label: "Hot corners".into(),
            enabled: true,
            children: corners,
        },
        Node::Sep,
        item("brightness", "Brightness..."),
        item("posh", "Terminal prompt..."),
        check(
            "click-highlight",
            "Highlight mouse clicks",
            true,
            c.click_highlight,
        ),
        check(
            "find-cursor",
            "Find cursor (press Ctrl twice)",
            true,
            c.find_cursor,
        ),
        check("keep-awake", "Keep system awake", true, c.keep_awake),
        check("startup", desktop.startup_label(), true, c.startup),
        item("brains", "Brains..."),
        item("check-update", "Check for updates..."),
        item("about", "About"),
        item("exit", "Exit"),
    ]
}

/// The logo at a tray size, as straight-alpha RGBA.
pub fn logo_rgba(size: u32) -> Vec<u8> {
    let mut out = vec![0u8; (size * size * 4) as usize];
    let decoder = png::Decoder::new(include_bytes!("../../../assets/logo.png").as_slice());
    let Ok(mut reader) = decoder.read_info() else {
        return out;
    };
    let mut buf = vec![0; reader.output_buffer_size()];
    let Ok(info) = reader.next_frame(&mut buf) else {
        return out;
    };
    let channels = info.color_type.samples();
    let (sw, sh) = (info.width as usize, info.height as usize);
    // A box filter, premultiplied so transparent pixels don't bleed into the edge.
    let s = size as usize;
    for y in 0..s {
        for x in 0..s {
            let (x0, x1) = (x * sw / s, ((x + 1) * sw / s).max(x * sw / s + 1));
            let (y0, y1) = (y * sh / s, ((y + 1) * sh / s).max(y * sh / s + 1));
            let (mut r, mut g, mut b, mut a, mut n) = (0u64, 0u64, 0u64, 0u64, 0u64);
            for yy in y0..y1 {
                for xx in x0..x1 {
                    let p = &buf[(yy * sw + xx) * channels..];
                    let (pr, pg, pb) = if channels >= 3 {
                        (p[0], p[1], p[2])
                    } else {
                        (p[0], p[0], p[0])
                    };
                    let pa = if channels == 4 {
                        p[3]
                    } else if channels == 2 {
                        p[1]
                    } else {
                        255
                    } as u64;
                    r += pr as u64 * pa;
                    g += pg as u64 * pa;
                    b += pb as u64 * pa;
                    a += pa;
                    n += 1;
                }
            }
            if a == 0 {
                continue; // fully transparent: leave the zeroed pixel
            }
            let i = (y * s + x) * 4;
            out[i] = (r / a) as u8;
            out[i + 1] = (g / a) as u8;
            out[i + 2] = (b / a) as u8;
            out[i + 3] = (a / n) as u8;
        }
    }
    out
}

// ---- Windows and macOS --------------------------------------------------------------

#[cfg(any(windows, target_os = "macos"))]
mod backend {
    use super::*;
    use tray_icon::{
        Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent,
    };

    pub struct Tray {
        _icon: TrayIcon,
    }

    impl Tray {
        pub fn new(proxy: EventLoopProxy<UserEvent>) -> Option<Tray> {
            TrayIconEvent::set_event_handler(Some(move |e: TrayIconEvent| {
                if let TrayIconEvent::Click {
                    button,
                    button_state: MouseButtonState::Up,
                    ..
                } = e
                {
                    let _ = match button {
                        // Control-click also arrives here as a left click.
                        MouseButton::Left if cfg!(target_os = "macos") => {
                            proxy.send_event(UserEvent::TrayLeftClick)
                        }
                        MouseButton::Left => proxy.send_event(UserEvent::TrayLeftClick),
                        MouseButton::Right => proxy.send_event(UserEvent::Menu("menu".into())),
                        _ => Ok(()),
                    };
                }
            }));
            let size = 32;
            let icon = Icon::from_rgba(logo_rgba(size), size, size).ok()?;
            let tray = TrayIconBuilder::new()
                .with_tooltip("Idle Walker")
                .with_icon(icon)
                .build()
                .ok()?;
            Some(Tray { _icon: tray })
        }
    }
}

// ---- Linux: StatusNotifierItem over D-Bus; no GTK needed ---------------------------------

#[cfg(target_os = "linux")]
mod backend {
    use super::*;
    use ksni::blocking::{Handle, TrayMethods};
    use ksni::menu::StandardItem;
    use ksni::MenuItem;

    struct Sni {
        proxy: EventLoopProxy<UserEvent>,
    }

    // The desktop draws this menu and closes it after every click, so it only leads
    // to the program's own settings menu, which stays open.
    fn entry(label: &str, id: &'static str) -> MenuItem<Sni> {
        StandardItem {
            label: label.into(),
            activate: Box::new(move |t: &mut Sni| {
                let _ = t.proxy.send_event(UserEvent::Menu(id.into()));
            }),
            ..Default::default()
        }
        .into()
    }

    impl ksni::Tray for Sni {
        fn id(&self) -> String {
            "idlewalker".into()
        }
        fn title(&self) -> String {
            "Idle Walker".into()
        }
        fn icon_pixmap(&self) -> Vec<ksni::Icon> {
            let size = 32;
            // The protocol wants ARGB, network byte order.
            let mut data = logo_rgba(size as u32);
            for p in data.as_chunks_mut::<4>().0 {
                p.rotate_right(1);
            }
            vec![ksni::Icon {
                width: size,
                height: size,
                data,
            }]
        }
        fn activate(&mut self, _x: i32, _y: i32) {
            let _ = self.proxy.send_event(UserEvent::TrayLeftClick);
        }
        fn menu(&self) -> Vec<MenuItem<Self>> {
            vec![
                entry("Settings...", "menu"),
                entry("Brightness...", "brightness"),
                entry("Terminal prompt...", "posh"),
                MenuItem::Separator,
                entry("Exit", "exit"),
            ]
        }
    }

    pub struct Tray {
        _handle: Handle<Sni>,
    }

    impl Tray {
        pub fn new(proxy: EventLoopProxy<UserEvent>) -> Option<Tray> {
            Sni { proxy }.spawn().ok().map(|h| Tray { _handle: h })
        }
    }
}

pub use backend::Tray;
