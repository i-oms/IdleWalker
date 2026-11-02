//! The rasterizer: background threads turning render snapshots into pixels, so the
//! UI thread never does pixel math. If a walker changes twice before its frame is
//! painted only the latest snapshot is kept, and when the painters fall behind frames
//! are skipped, never waited for. A walker is painted by one painter at a time, so its
//! frames always come back in order. A `Surface` is owned by exactly one thread at a
//! time: the pool hands it to a painter, the result hands it back.
use crate::prelude::*;
use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Instant;

/// Room for every canvas the full cast can have in flight at once.
const POOL_CAP: usize = 32;

/// Painter threads: two where there are cores to spare, since a Retina frame is four
/// times the pixels and one painter cannot keep a full cast at the refresh rate.
fn painter_count() -> usize {
    match std::thread::available_parallelism().map_or(1, |n| n.get()) {
        n if n >= 4 => 2,
        _ => 1,
    }
}

/// A finished frame, ready to show.
pub struct PaintResult {
    pub walker: i32,
    pub stamp: u32,
    pub surface: Surface,
    pub left: i32,
    pub top: i32,
}

#[derive(Default)]
struct Shared {
    queue: VecDeque<usize>,
    slots: Vec<Option<RenderJob>>,
    waiting: Vec<bool>,
    /// Being painted right now: a newer snapshot waits until it is done.
    busy: Vec<bool>,
    done: VecDeque<PaintResult>,
    pool: Vec<Surface>,
    stopping: bool,
    paint_secs: f64,
}

#[derive(Default)]
pub struct RenderThread {
    gate: Arc<(Mutex<Shared>, Condvar)>,
    threads: Vec<std::thread::JoinHandle<()>>,
}

impl RenderThread {
    /// Starts the painters. `on_painter_start` runs first on each painter thread; the
    /// composition root uses it to lower the thread's priority, which is an OS matter.
    pub fn start(&mut self, on_painter_start: fn()) {
        for i in 0..painter_count() {
            let gate = self.gate.clone();
            let t = std::thread::Builder::new()
                .name(format!("IdleWalker paint {}", i + 1))
                .spawn(move || {
                    // Pixels are never urgent: the painters yield to whatever the user is doing.
                    on_painter_start();
                    paint_loop(gate);
                });
            if let Ok(t) = t {
                self.threads.push(t);
            }
        }
    }

    /// The share of a core the painters may use between them, for the paint budget.
    pub fn budget(&self) -> f64 {
        Settings::PAINT_BUDGET * self.threads.len().max(1) as f64
    }

    /// Milliseconds spent painting since the last call, for the paint budget.
    pub fn take_paint_ms(&self) -> f64 {
        let mut s = self.gate.0.lock().unwrap();
        let v = s.paint_secs * 1000.0;
        s.paint_secs = 0.0;
        v
    }

    /// Queues a snapshot; a newer one for the same walker wins.
    pub fn request(&self, walker: usize, job: RenderJob) {
        let mut s = self.gate.0.lock().unwrap();
        if s.stopping {
            return;
        }
        if walker >= s.slots.len() {
            s.slots.resize_with(walker + 4, || None);
            s.waiting.resize(walker + 4, false);
            s.busy.resize(walker + 4, false);
        }
        s.slots[walker] = Some(job);
        if !s.waiting[walker] && !s.busy[walker] {
            s.waiting[walker] = true;
            s.queue.push_back(walker);
            self.gate.1.notify_one();
        }
    }

    /// Takes a finished frame, if any.
    pub fn try_take(&self) -> Option<PaintResult> {
        self.gate.0.lock().unwrap().done.pop_front()
    }

    /// Returns a shown surface to the pool.
    pub fn release(&self, surface: Surface) {
        let mut s = self.gate.0.lock().unwrap();
        if s.pool.len() < POOL_CAP {
            s.pool.push(surface);
        } else {
            surface.free();
        }
    }

    pub fn stop(&mut self) {
        self.gate.0.lock().unwrap().stopping = true;
        self.gate.1.notify_all();
        for t in self.threads.drain(..) {
            let _ = t.join();
        }
    }
}

fn paint_loop(gate: Arc<(Mutex<Shared>, Condvar)>) {
    loop {
        let (w, mut job, pooled) = {
            let mut s = gate.0.lock().unwrap();
            while !s.stopping && s.queue.is_empty() {
                s = gate.1.wait(s).unwrap();
            }
            if s.stopping {
                return;
            }
            let w = s.queue.pop_front().unwrap();
            s.waiting[w] = false;
            s.busy[w] = true;
            let job = s.slots[w].take().unwrap();
            (w, job, s.pool.pop())
        };
        let surface = pooled.unwrap_or_else(Surface::new);
        let started = Instant::now();
        // One bad frame must never kill the painter.
        let ok = rt::attempt(|| FrameRenderer::paint_job(&mut job, surface));
        let spent = started.elapsed().as_secs_f64();
        let mut s = gate.0.lock().unwrap();
        s.busy[w] = false;
        // A snapshot that arrived while this one was painted goes next.
        if s.slots[w].is_some() && !s.waiting[w] {
            s.waiting[w] = true;
            s.queue.push_back(w);
            gate.1.notify_one();
        }
        if ok {
            s.paint_secs += spent;
            s.done.push_back(PaintResult {
                walker: w as i32,
                stamp: job.stamp,
                surface,
                left: job.left,
                top: job.top,
            });
        } else if s.pool.len() < POOL_CAP {
            s.pool.push(surface);
        }
    }
}
