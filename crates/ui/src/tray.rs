//! The tray icon and its menu, grouped by feature: Walkers (the master switch plus the
//! shared "Appear after" delay), then the cast, then Hot corners, then the program.
//! The menu is described as plain data from the current choices and shown by
//! menu_panel.rs. The icon itself comes from `tray-icon` on Windows and macOS, and on
//! Linux from the StatusNotifierItem D-Bus protocol or, failing that, the XEmbed system
//! tray. A left-click opens the brightness panel.
use super::UserEvent;
use crate::prelude::*;
pub use idlewalker_application::Choices;
use idlewalker_application::{Desktop, Indicator, Modifier};
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

/// The colours offered for the cursor indicators: bright ones, to show on any screen.
const INDICATOR_COLOURS: [(&str, i32); 9] = [
    ("Yellow", 0xFFE100),
    ("Blue", 0x2D7DFF),
    ("Red", 0xFF3B30),
    ("Green", 0x34C759),
    ("Orange", 0xFF9500),
    ("Pink", 0xFF2D95),
    ("Purple", 0xAF52DE),
    ("Cyan", 0x32C8E6),
    ("White", 0xFFFFFF),
];

/// A group for choosing one indicator's colour: "Left click: Yellow", say.
fn indicator_colour(c: &Choices, which: Indicator, label: &str, enabled: bool) -> Node {
    let chosen = c.indicator_colours[which as usize];
    let named = INDICATOR_COLOURS.iter().find(|(_, rgb)| *rgb == chosen);
    let mut children: Vec<Node> = INDICATOR_COLOURS
        .iter()
        .map(|(name, rgb)| {
            check(
                format!("{}:{}", which.id(), rgb),
                *name,
                true,
                *rgb == chosen,
            )
        })
        .collect();
    children.push(Node::Sep);
    children.push(check(
        format!("{}-custom", which.id()),
        "Custom...",
        true,
        named.is_none(),
    ));
    let shown = match named {
        Some((name, _)) => name.to_string(),
        None => format!("Custom #{:06X}", chosen),
    };
    Node::Sub {
        label: format!("{}: {}", label, shown),
        enabled,
        children,
    }
}

/// The whole menu, as the choices stand. A feature's settings are greyed while it is off;
/// group headers stay enabled so a group can always be opened.
pub fn build(c: &Choices, desktop: &dyn Desktop) -> Vec<Node> {
    let action_names = desktop.corner_action_names();
    let w = c.walkers;
    // The features that follow the cursor, greyed where it cannot be followed (Wayland).
    let tracked = desktop.pointer_tracked();
    let needs_cursor = |label: String| {
        if tracked {
            label
        } else {
            format!("{} (unavailable)", label)
        }
    };

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

    let mut find = vec![
        check("find-cursor", "Enabled", true, c.find_cursor),
        Node::Sep,
    ];
    for key in Modifier::ALL {
        find.push(check(
            format!("find-key:{}", key as i32),
            format!("Press {} twice", desktop.modifier_name(key)),
            c.find_cursor,
            key == c.find_key,
        ));
    }
    find.push(Node::Sep);
    find.push(indicator_colour(
        c,
        Indicator::FindRing,
        "Ring colour",
        c.find_cursor,
    ));

    let clicks = vec![
        check("click-highlight", "Enabled", true, c.click_highlight),
        Node::Sep,
        indicator_colour(c, Indicator::LeftClick, "Left click", c.click_highlight),
        indicator_colour(c, Indicator::RightClick, "Right click", c.click_highlight),
    ];

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
            label: needs_cursor("Hot corners".into()),
            enabled: tracked,
            children: corners,
        },
        Node::Sep,
        item("brightness", "Brightness..."),
        item("posh", "Terminal prompt..."),
        Node::Sub {
            label: needs_cursor("Highlight mouse clicks".into()),
            enabled: tracked,
            children: clicks,
        },
        Node::Sub {
            label: needs_cursor(format!(
                "Find cursor: {} twice",
                desktop.modifier_name(c.find_key)
            )),
            enabled: tracked,
            children: find,
        },
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

        /// The notification area and the menu bar are always there to take it.
        pub fn shown(&self) -> bool {
            true
        }
    }
}

// ---- Linux: StatusNotifierItem over D-Bus, else an XEmbed tray icon; no GTK needed ----------
//
// Linux desktops disagree on the tray. KDE, Cinnamon, Budgie, newer Xfce and GNOME with
// the AppIndicator extension host StatusNotifierItems over D-Bus. Xfce's older panels,
// MATE, LXDE and most bare window managers' bars only take the XEmbed system tray.
// Plain GNOME has neither; there a second start of the program opens its menu instead.

#[cfg(target_os = "linux")]
mod backend {
    use super::*;
    use ksni::blocking::{Handle, TrayMethods};
    use ksni::menu::StandardItem;
    use ksni::MenuItem;

    /// The icon at the sizes trays ask for, so none has to scale a 32 pixel one.
    const ICON_SIZES: [u32; 6] = [16, 22, 24, 32, 48, 64];

    struct Sni {
        proxy: EventLoopProxy<UserEvent>,
        icons: Vec<ksni::Icon>,
    }

    impl Sni {
        fn new(proxy: EventLoopProxy<UserEvent>) -> Sni {
            let icons = ICON_SIZES
                .iter()
                .map(|&size| {
                    // The protocol wants ARGB, network byte order.
                    let mut data = logo_rgba(size);
                    for p in data.as_chunks_mut::<4>().0 {
                        p.rotate_right(1);
                    }
                    ksni::Icon {
                        width: size as i32,
                        height: size as i32,
                        data,
                    }
                })
                .collect();
            Sni { proxy, icons }
        }
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
        fn tool_tip(&self) -> ksni::ToolTip {
            ksni::ToolTip {
                title: "Idle Walker".into(),
                ..Default::default()
            }
        }
        fn icon_pixmap(&self) -> Vec<ksni::Icon> {
            self.icons.clone()
        }
        fn activate(&mut self, _x: i32, _y: i32) {
            let _ = self.proxy.send_event(UserEvent::TrayLeftClick);
        }
        /// A middle click.
        fn secondary_activate(&mut self, _x: i32, _y: i32) {
            let _ = self.proxy.send_event(UserEvent::Menu("menu".into()));
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
        _sni: Option<Handle<Sni>>,
        shown: bool,
    }

    impl Tray {
        pub fn new(proxy: EventLoopProxy<UserEvent>) -> Option<Tray> {
            // A StatusNotifierItem, where something is there to show it.
            if let Ok(h) = Sni::new(proxy.clone()).spawn() {
                return Some(Tray {
                    _sni: Some(h),
                    shown: true,
                });
            }
            // The XEmbed system tray.
            if xembed::start(proxy.clone()) {
                return Some(Tray {
                    _sni: None,
                    shown: true,
                });
            }
            // Neither yet. Started with the session, the program can be up before the
            // panel is: the item shows as soon as a StatusNotifierItem host comes.
            let h = Sni::new(proxy).assume_sni_available(true).spawn().ok();
            Some(Tray {
                _sni: h,
                shown: false,
            })
        }

        /// Whether a tray took the icon. False while it waits for one that may never come.
        pub fn shown(&self) -> bool {
            self.shown
        }
    }

    /// The XEmbed system tray (freedesktop's System Tray Protocol): a small window of our
    /// own that the panel takes in, drawn here and listening for clicks.
    mod xembed {
        use super::super::logo_rgba;
        use crate::UserEvent;
        use winit::event_loop::EventLoopProxy;
        use x11rb::connection::Connection;
        use x11rb::protocol::xproto::*;
        use x11rb::protocol::Event;
        use x11rb::rust_connection::RustConnection;
        use x11rb::wrapper::ConnectionExt as _;
        use x11rb::{CURRENT_TIME, NONE};

        const SYSTEM_TRAY_REQUEST_DOCK: u32 = 0;
        /// XEMBED_MAPPED: map the icon once it is taken in.
        const XEMBED_MAPPED: u32 = 1;

        struct Atoms {
            selection: Atom,
            opcode: Atom,
            manager: Atom,
            visual: Atom,
            xembed_info: Atom,
        }

        fn atom(c: &RustConnection, name: &str) -> Atom {
            c.intern_atom(false, name.as_bytes())
                .ok()
                .and_then(|r| r.reply().ok())
                .map_or(NONE, |r| r.atom)
        }

        struct Icon {
            conn: RustConnection,
            screen: usize,
            atoms: Atoms,
            window: Window,
            gc: Gcontext,
            /// Drawn with per-pixel alpha; otherwise over a copy of the panel behind it.
            argb: bool,
            depth: u8,
            size: (u16, u16),
        }

        /// Docks the icon in the tray and serves it from a thread of its own; false when
        /// no XEmbed tray is running.
        pub fn start(proxy: EventLoopProxy<UserEvent>) -> bool {
            let Ok((conn, screen)) = RustConnection::connect(None) else {
                return false;
            };
            let atoms = Atoms {
                selection: atom(&conn, &format!("_NET_SYSTEM_TRAY_S{}", screen)),
                opcode: atom(&conn, "_NET_SYSTEM_TRAY_OPCODE"),
                manager: atom(&conn, "MANAGER"),
                visual: atom(&conn, "_NET_SYSTEM_TRAY_VISUAL"),
                xembed_info: atom(&conn, "_XEMBED_INFO"),
            };
            let Some(owner) = tray_owner(&conn, &atoms) else {
                return false;
            };
            let mut icon = Icon {
                conn,
                screen,
                atoms,
                window: NONE,
                gc: NONE,
                argb: false,
                depth: 0,
                size: (24, 24),
            };
            if !icon.dock(owner) {
                return false;
            }
            std::thread::Builder::new()
                .name("IdleWalker tray".into())
                .spawn(move || icon.serve(proxy))
                .is_ok()
        }

        fn tray_owner(conn: &RustConnection, atoms: &Atoms) -> Option<Window> {
            if atoms.selection == NONE {
                return None;
            }
            let owner = conn
                .get_selection_owner(atoms.selection)
                .ok()?
                .reply()
                .ok()?
                .owner;
            (owner != NONE).then_some(owner)
        }

        impl Icon {
            fn root(&self) -> &Screen {
                &self.conn.setup().roots[self.screen]
            }

            /// The 32-bit visual the tray asks icons to use, when it has one.
            fn tray_visual(&self, owner: Window) -> Option<Visualid> {
                let id = self
                    .conn
                    .get_property(false, owner, self.atoms.visual, AtomEnum::VISUALID, 0, 1)
                    .ok()?
                    .reply()
                    .ok()?
                    .value32()?
                    .next()?;
                self.root()
                    .allowed_depths
                    .iter()
                    .filter(|d| d.depth == 32)
                    .flat_map(|d| d.visuals.iter())
                    .any(|v| v.visual_id == id)
                    .then_some(id)
            }

            /// Makes a new icon window and asks the tray that owns the selection to take it.
            fn dock(&mut self, owner: Window) -> bool {
                let c = &self.conn;
                let root = self.root().root;
                if self.window != NONE {
                    let _ = c.destroy_window(self.window);
                    self.window = NONE;
                }
                let Ok(window) = c.generate_id() else {
                    return false;
                };
                let events = EventMask::EXPOSURE
                    | EventMask::STRUCTURE_NOTIFY
                    | EventMask::BUTTON_PRESS
                    | EventMask::BUTTON_RELEASE;
                let (w, h) = self.size;
                let made = match self.tray_visual(owner) {
                    Some(visual) => {
                        self.argb = true;
                        self.depth = 32;
                        let Ok(cm) = c.generate_id() else {
                            return false;
                        };
                        let _ = c.create_colormap(ColormapAlloc::NONE, cm, root, visual);
                        let aux = CreateWindowAux::new()
                            .background_pixel(0)
                            .border_pixel(0)
                            .colormap(cm)
                            .event_mask(events);
                        c.create_window(
                            32,
                            window,
                            root,
                            0,
                            0,
                            w,
                            h,
                            0,
                            WindowClass::INPUT_OUTPUT,
                            visual,
                            &aux,
                        )
                    }
                    None => {
                        self.argb = false;
                        self.depth = self.root().root_depth;
                        let aux = CreateWindowAux::new()
                            .background_pixmap(BackPixmap::PARENT_RELATIVE)
                            .event_mask(events);
                        c.create_window(
                            x11rb::COPY_DEPTH_FROM_PARENT,
                            window,
                            root,
                            0,
                            0,
                            w,
                            h,
                            0,
                            WindowClass::INPUT_OUTPUT,
                            x11rb::COPY_FROM_PARENT,
                            &aux,
                        )
                    }
                };
                if made.is_err() {
                    return false;
                }
                self.window = window;
                if self.gc != NONE {
                    let _ = c.free_gc(self.gc);
                }
                if let Ok(gc) = c.generate_id() {
                    let aux = CreateGCAux::new().graphics_exposures(0);
                    if c.create_gc(gc, window, &aux).is_ok() {
                        self.gc = gc;
                    }
                }
                let _ = c.change_property32(
                    PropMode::REPLACE,
                    window,
                    self.atoms.xembed_info,
                    self.atoms.xembed_info,
                    &[0, XEMBED_MAPPED],
                );
                let _ = c.change_property8(
                    PropMode::REPLACE,
                    window,
                    AtomEnum::WM_NAME,
                    AtomEnum::STRING,
                    b"Idle Walker",
                );
                let _ = c.change_property8(
                    PropMode::REPLACE,
                    window,
                    AtomEnum::WM_CLASS,
                    AtomEnum::STRING,
                    b"idlewalker\0IdleWalker\0",
                );
                let ask = ClientMessageEvent::new(
                    32,
                    owner,
                    self.atoms.opcode,
                    [CURRENT_TIME, SYSTEM_TRAY_REQUEST_DOCK, window, 0, 0],
                );
                let _ = c.send_event(false, owner, EventMask::NO_EVENT, ask);
                c.flush().is_ok()
            }

            /// Answers the tray until the X server goes away.
            fn serve(mut self, proxy: EventLoopProxy<UserEvent>) {
                // A tray that starts later announces itself on the root window.
                let root = self.root().root;
                let _ = self.conn.change_window_attributes(
                    root,
                    &ChangeWindowAttributesAux::new().event_mask(EventMask::STRUCTURE_NOTIFY),
                );
                let _ = self.conn.flush();
                while let Ok(event) = self.conn.wait_for_event() {
                    match event {
                        Event::Expose(e) if e.window == self.window && e.count == 0 => self.draw(),
                        Event::ConfigureNotify(e) if e.window == self.window => {
                            let size = (e.width.max(1), e.height.max(1));
                            if size != self.size {
                                self.size = size;
                                self.draw();
                            }
                        }
                        // The tray went away and handed the icon back: keep it off the desktop.
                        Event::ReparentNotify(e) if e.window == self.window && e.parent == root => {
                            let _ = self.conn.unmap_window(self.window);
                            let _ = self.conn.flush();
                        }
                        Event::ButtonRelease(e) if e.event == self.window => {
                            let sent = match e.detail {
                                1 => proxy.send_event(UserEvent::TrayLeftClick),
                                2 | 3 => proxy.send_event(UserEvent::Menu("menu".into())),
                                _ => Ok(()),
                            };
                            if sent.is_err() {
                                return; // the program is closing
                            }
                        }
                        Event::ClientMessage(e)
                            if e.window == root
                                && e.type_ == self.atoms.manager
                                && e.data.as_data32()[1] == self.atoms.selection =>
                        {
                            let owner = e.data.as_data32()[2];
                            if owner != NONE {
                                self.dock(owner);
                            }
                        }
                        _ => {}
                    }
                }
            }

            /// Paints the logo, as large as fits and centred.
            fn draw(&self) {
                let c = &self.conn;
                let (w, h) = (self.size.0 as usize, self.size.1 as usize);
                // Only the common layout: four bytes a pixel, blue first.
                let four_bytes = c
                    .setup()
                    .pixmap_formats
                    .iter()
                    .any(|f| f.depth == self.depth && f.bits_per_pixel == 32);
                if !four_bytes || c.setup().image_byte_order != ImageOrder::LSB_FIRST {
                    return;
                }
                let side = w.min(h).max(1);
                let logo = logo_rgba(side as u32);
                let (ox, oy) = ((w - side) / 2, (h - side) / 2);
                // What shows through: nothing with alpha, the panel without it.
                let mut out = vec![0u8; w * h * 4];
                if !self.argb {
                    let _ = c.clear_area(false, self.window, 0, 0, 0, 0);
                    let behind = c
                        .get_image(
                            ImageFormat::Z_PIXMAP,
                            self.window,
                            0,
                            0,
                            w as u16,
                            h as u16,
                            !0,
                        )
                        .ok()
                        .and_then(|r| r.reply().ok())
                        .filter(|r| r.data.len() >= out.len());
                    match behind {
                        Some(r) => {
                            let n = out.len();
                            out.copy_from_slice(&r.data[..n])
                        }
                        None => out.fill(0x30),
                    }
                }
                for y in 0..side {
                    for x in 0..side {
                        let s = &logo[(y * side + x) * 4..][..4];
                        let a = s[3] as u32;
                        let d = &mut out[((oy + y) * w + ox + x) * 4..][..4];
                        // Logo is RGBA; the image is BGRA, premultiplied where it has alpha.
                        for (k, from) in [(0, 2), (1, 1), (2, 0)] {
                            d[k] = ((s[from] as u32 * a + d[k] as u32 * (255 - a)) / 255) as u8;
                        }
                        if self.argb {
                            d[3] = (a + d[3] as u32 * (255 - a) / 255) as u8;
                        }
                    }
                }
                let _ = c.put_image(
                    ImageFormat::Z_PIXMAP,
                    self.window,
                    self.gc,
                    w as u16,
                    h as u16,
                    0,
                    0,
                    0,
                    self.depth,
                    &out,
                );
                let _ = c.flush();
            }
        }
    }
}

pub use backend::Tray;
