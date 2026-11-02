//! The small runtime under the translated simulation and painters: C#'s
//! reference semantics, .NET's numeric behaviour, and its `Random`.
//!
//! The cast is an object graph - walkers hold each other and call each other
//! re-entrantly - so a class becomes a `Copy` handle to heap data. Methods take
//! the handle by value and field accesses are momentary, so no Rust reference
//! outlives one expression. A handle is only ever used from one thread at a
//! time (the UI thread owns the cast, the painter owns its own painters).
#![allow(clippy::all)]
use std::cell::UnsafeCell;
use std::fmt::Display;
use std::ops::Index;
use std::sync::{Arc, OnceLock};

/// Declares the handle type of a translated class.
#[macro_export]
macro_rules! handle {
    ($h:ident, $d:ident) => {
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $h(pub *mut $d);
        unsafe impl Send for $h {}
        unsafe impl Sync for $h {}
        impl Default for $h {
            fn default() -> Self {
                $h(std::ptr::null_mut())
            }
        }
        impl $h {
            #[inline]
            pub fn is_null(&self) -> bool {
                self.0.is_null()
            }
            pub fn from_data(d: $d) -> Self {
                $h(Box::into_raw(Box::new(d)))
            }
            /// Releases an object C# would have left to the collector.
            pub fn free(self) {
                if !self.0.is_null() {
                    unsafe {
                        drop(Box::from_raw(self.0));
                    }
                }
            }
        }
        impl std::ops::Deref for $h {
            type Target = $d;
            #[inline]
            fn deref(&self) -> &$d {
                unsafe { &*self.0 }
            }
        }
        impl std::ops::DerefMut for $h {
            #[inline]
            fn deref_mut(&mut self) -> &mut $d {
                unsafe { &mut *self.0 }
            }
        }
    };
}
pub use crate::handle;

// ---- Arrays and lists (reference types, as in C#) ------------------------

/// `T[]`, `List<T>`, `Queue<T>`: a shared, growable buffer, indexed by i32.
pub struct Arr<T>(Option<Arc<UnsafeCell<Vec<T>>>>);
unsafe impl<T> Send for Arr<T> {}
unsafe impl<T> Sync for Arr<T> {}
impl<T> Clone for Arr<T> {
    #[inline]
    fn clone(&self) -> Self {
        Arr(self.0.clone())
    }
}
impl<T> Default for Arr<T> {
    fn default() -> Self {
        Arr(None)
    }
}
impl<T> PartialEq for Arr<T> {
    fn eq(&self, o: &Self) -> bool {
        match (&self.0, &o.0) {
            (None, None) => true,
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            _ => false,
        }
    }
}
impl<T> Arr<T> {
    #[inline]
    fn v(&self) -> &mut Vec<T> {
        unsafe { &mut *self.0.as_ref().expect("null array").get() }
    }
    pub fn from_vec(v: Vec<T>) -> Self {
        Arr(Some(Arc::new(UnsafeCell::new(v))))
    }
    #[inline]
    pub fn is_null(&self) -> bool {
        self.0.is_none()
    }
    #[inline]
    pub fn len(&self) -> i32 {
        self.v().len() as i32
    }
    #[inline]
    pub fn at(&self, i: i32) -> &mut T {
        &mut self.v()[i as usize]
    }
    pub fn add(&self, x: T) {
        self.v().push(x)
    }
    pub fn clear(&self) {
        self.v().clear()
    }
    pub fn remove_at(&self, i: i32) {
        self.v().remove(i as usize);
    }
    pub fn as_slice(&self) -> &[T] {
        self.v().as_slice()
    }
    pub fn as_mut_slice(&self) -> &mut [T] {
        self.v().as_mut_slice()
    }
}
impl<T: Default + Clone> Arr<T> {
    pub fn new(n: i32) -> Self {
        Self::from_vec(vec![T::default(); n as usize])
    }
    pub fn clear_range(&self, start: i32, n: i32) {
        for x in &mut self.v()[start as usize..(start + n) as usize] {
            *x = T::default();
        }
    }
}
impl<T: Clone> Arr<T> {
    pub fn items(&self) -> std::vec::IntoIter<T> {
        self.v().clone().into_iter()
    }
    pub fn deep_copy(&self) -> Self {
        Self::from_vec(self.v().clone())
    }
}
impl<T: PartialEq> Arr<T> {
    pub fn contains(&self, x: T) -> bool {
        self.v().contains(&x)
    }
}
impl<T> Index<i32> for Arr<T> {
    type Output = T;
    #[inline]
    fn index(&self, i: i32) -> &T {
        &self.v()[i as usize]
    }
}

/// `T[,]`.
pub struct Arr2<T> {
    a: Arr<T>,
    rows: i32,
    cols: i32,
}
impl<T> Clone for Arr2<T> {
    fn clone(&self) -> Self {
        Arr2 {
            a: self.a.clone(),
            rows: self.rows,
            cols: self.cols,
        }
    }
}
impl<T> Default for Arr2<T> {
    fn default() -> Self {
        Arr2 {
            a: Arr::default(),
            rows: 0,
            cols: 0,
        }
    }
}
impl<T> Arr2<T> {
    #[inline]
    pub fn is_null(&self) -> bool {
        self.a.is_null()
    }
    #[inline]
    pub fn at2(&self, r: i32, c: i32) -> &mut T {
        assert!(c >= 0 && c < self.cols && r >= 0 && r < self.rows);
        self.a.at(r * self.cols + c)
    }
    pub fn get_length(&self, d: i32) -> i32 {
        if d == 0 {
            self.rows
        } else {
            self.cols
        }
    }
    pub fn len(&self) -> i32 {
        self.rows * self.cols
    }
}
impl<T: Default + Clone> Arr2<T> {
    pub fn new(r: i32, c: i32) -> Self {
        Arr2 {
            a: Arr::new(r * c),
            rows: r,
            cols: c,
        }
    }
    pub fn from_rows(rows: Vec<Vec<T>>) -> Self {
        let (r, c) = (
            rows.len() as i32,
            rows.first().map_or(0, |x| x.len()) as i32,
        );
        Arr2 {
            a: Arr::from_vec(rows.into_iter().flatten().collect()),
            rows: r,
            cols: c,
        }
    }
    pub fn items(&self) -> std::vec::IntoIter<T> {
        self.a.items()
    }
    pub fn deep_copy(&self) -> Self {
        Arr2 {
            a: self.a.deep_copy(),
            rows: self.rows,
            cols: self.cols,
        }
    }
}

pub fn array_copy<T: Clone>(src: &Arr<T>, si: i32, dst: &Arr<T>, di: i32, n: i32) {
    let tmp: Vec<T> = src.as_slice()[si as usize..(si + n) as usize].to_vec();
    dst.as_mut_slice()[di as usize..(di + n) as usize].clone_from_slice(&tmp);
}
pub fn array_resize<T: Default + Clone>(a: &mut Arr<T>, n: i32) {
    let mut v: Vec<T> = if a.is_null() {
        Vec::new()
    } else {
        a.as_slice().to_vec()
    };
    v.resize(n as usize, T::default());
    *a = Arr::from_vec(v);
}

// ---- Dictionaries: insertion ordered, so runs are reproducible -------------

#[derive(Clone, Default)]
pub struct Kv<K, V> {
    pub key: K,
    pub value: V,
}

pub struct Dict<K, V>(Arr<Kv<K, V>>);
impl<K, V> Clone for Dict<K, V> {
    fn clone(&self) -> Self {
        Dict(self.0.clone())
    }
}
impl<K, V> Default for Dict<K, V> {
    fn default() -> Self {
        Dict(Arr::default())
    }
}
impl<K: PartialEq + Clone, V: Clone> Dict<K, V> {
    pub fn new() -> Self {
        Dict(Arr::from_vec(Vec::new()))
    }
    pub fn is_null(&self) -> bool {
        self.0.is_null()
    }
    pub fn len(&self) -> i32 {
        self.0.len()
    }
    pub fn clear(&self) {
        self.0.clear()
    }
    pub fn try_get_value(&self, k: &K, out: &mut V) -> bool {
        for e in self.0.as_slice() {
            if e.key == *k {
                *out = e.value.clone();
                return true;
            }
        }
        false
    }
    pub fn contains_key(&self, k: &K) -> bool {
        self.0.as_slice().iter().any(|e| e.key == *k)
    }
    pub fn get(&self, k: &K) -> V {
        self.0
            .as_slice()
            .iter()
            .find(|e| e.key == *k)
            .expect("no such key")
            .value
            .clone()
    }
    pub fn set(&self, k: K, v: V) {
        for e in self.0.as_mut_slice() {
            if e.key == k {
                e.value = v;
                return;
            }
        }
        self.0.add(Kv { key: k, value: v });
    }
    pub fn remove(&self, k: &K) -> bool {
        match self.0.as_slice().iter().position(|e| e.key == *k) {
            Some(i) => {
                self.0.remove_at(i as i32);
                true
            }
            None => false,
        }
    }
    pub fn items(&self) -> std::vec::IntoIter<Kv<K, V>> {
        self.0.items()
    }
    pub fn keys(&self) -> Arr<K> {
        Arr::from_vec(self.0.as_slice().iter().map(|e| e.key.clone()).collect())
    }
}

// ---- Statics ---------------------------------------------------------------

/// A C# `static readonly`: built on first use.
pub struct Global<T> {
    cell: OnceLock<GlobalBox<T>>,
    init: fn() -> T,
}
struct GlobalBox<T>(UnsafeCell<T>);
unsafe impl<T> Send for GlobalBox<T> {}
unsafe impl<T> Sync for GlobalBox<T> {}
impl<T> Global<T> {
    pub const fn new(init: fn() -> T) -> Self {
        Global {
            cell: OnceLock::new(),
            init,
        }
    }
    /// A mutable C# static: only ever touched from the thread that owns the cast.
    pub fn at(&self) -> &mut T {
        unsafe {
            &mut *self
                .cell
                .get_or_init(|| GlobalBox(UnsafeCell::new((self.init)())))
                .0
                .get()
        }
    }
}
impl<T> std::ops::Deref for Global<T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        unsafe {
            &*self
                .cell
                .get_or_init(|| GlobalBox(UnsafeCell::new((self.init)())))
                .0
                .get()
        }
    }
}

// ---- .NET numerics ----------------------------------------------------------

#[inline]
pub fn min_f64(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else if a < b {
        a
    } else {
        b
    }
}
#[inline]
pub fn max_f64(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else if a > b {
        a
    } else {
        b
    }
}
#[inline]
pub fn min_i32(a: i32, b: i32) -> i32 {
    a.min(b)
}
#[inline]
pub fn max_i32(a: i32, b: i32) -> i32 {
    a.max(b)
}
#[inline]
pub fn sign_f64(a: f64) -> i32 {
    if a > 0.0 {
        1
    } else if a < 0.0 {
        -1
    } else {
        0
    }
}
#[inline]
pub fn sign_i32(a: i32) -> i32 {
    a.signum()
}
#[inline]
pub fn pow(a: f64, b: f64) -> f64 {
    a.powf(b)
}
/// `Math.Round`: halves go to the even neighbour.
#[inline]
pub fn round_even(x: f64) -> f64 {
    x.round_ties_even()
}
pub fn round_digits(x: f64, d: i32) -> f64 {
    let p = 10f64.powi(d);
    (x * p).round_ties_even() / p
}

pub fn cat<A: Display, B: Display>(a: A, b: B) -> String {
    format!("{}{}", a, b)
}
pub fn parse_i32(s: &str) -> i32 {
    s.trim().parse().expect("int")
}
pub fn parse_f64(s: &str) -> f64 {
    s.trim().parse().expect("double")
}
pub fn try_parse_i32(s: &str, out: &mut i32) -> bool {
    match s.trim().parse() {
        Ok(v) => {
            *out = v;
            true
        }
        Err(_) => {
            *out = 0;
            false
        }
    }
}
/// Parses a double the way the invariant culture does, NaN and infinities included.
pub fn try_parse_f64(s: &str, out: &mut f64) -> bool {
    let t = s.trim();
    let r = match t {
        "NaN" => Ok(f64::NAN),
        "Infinity" => Ok(f64::INFINITY),
        "-Infinity" => Ok(f64::NEG_INFINITY),
        _ => t.parse(),
    };
    match r {
        Ok(v) => {
            *out = v;
            true
        }
        Err(_) => {
            *out = 0.0;
            false
        }
    }
}
/// `double.ToString()`, shortest round-trip.
pub fn num_to_string(x: f64) -> String {
    format!("{}", x)
}
/// `x.ToString(format)` for the formats the program uses: "R" and "0.00"-style.
pub fn format_num(x: f64, f: &str) -> String {
    if f == "R" || f == "r" || f.is_empty() {
        return num_to_string(x);
    }
    if let Some(dot) = f.find('.') {
        let frac = &f[dot + 1..];
        if frac.chars().all(|c| c == '#') {
            // up to n decimals, trailing zeros dropped
            let s = format!("{:.*}", frac.len(), x);
            let s = if s.contains('.') {
                s.trim_end_matches('0').trim_end_matches('.').to_string()
            } else {
                s
            };
            return if s == "-0" { "0".to_string() } else { s };
        }
        return format!("{:.*}", frac.len(), x);
    }
    if let Some(rest) = f.strip_prefix('F') {
        return format!("{:.*}", rest.parse().unwrap_or(2), x);
    }
    if let Some(rest) = f.strip_prefix('N') {
        return format!("{:.*}", rest.parse().unwrap_or(2), x);
    }
    format!("{:.0}", x)
}

pub fn split(s: &str, sep: char) -> Arr<String> {
    Arr::from_vec(s.split(sep).map(|x| x.to_string()).collect())
}
pub fn join(sep: &str, a: &Arr<String>) -> String {
    a.as_slice().join(sep)
}
pub fn substr(s: &str, start: i32, n: i32) -> String {
    let it = s.chars().skip(start as usize);
    if n < 0 {
        it.collect()
    } else {
        it.take(n as usize).collect()
    }
}
pub trait Needle {
    fn find_in(&self, s: &str) -> Option<usize>;
}
impl Needle for char {
    fn find_in(&self, s: &str) -> Option<usize> {
        s.find(*self)
    }
}
impl Needle for String {
    fn find_in(&self, s: &str) -> Option<usize> {
        s.find(self.as_str())
    }
}
pub fn index_of<N: Needle>(s: &str, n: N) -> i32 {
    n.find_in(s).map_or(-1, |b| s[..b].chars().count() as i32)
}
pub fn char_at(s: &str, i: i32) -> char {
    s.chars().nth(i as usize).expect("index")
}

/// An `out` argument: lets go of the borrow of the handle it was reached through,
/// as field accesses through a handle are momentary (see the top of this file).
#[inline]
pub fn mref<'a, T>(x: &mut T) -> &'a mut T {
    unsafe { &mut *(x as *mut T) }
}

/// `try { } catch (Exception) { }`: true when the body finished.
pub fn attempt<F: FnOnce()>(f: F) -> bool {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).is_ok()
}

/// The same, for a body with a result.
pub fn attempt_val<R, F: FnOnce() -> R>(f: F) -> Option<R> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).ok()
}

// ---- Time --------------------------------------------------------------------

static START: OnceLock<std::time::Instant> = OnceLock::new();
pub fn stopwatch_timestamp() -> i64 {
    START
        .get_or_init(std::time::Instant::now)
        .elapsed()
        .as_nanos() as i64
}
pub fn stopwatch_frequency() -> i64 {
    1_000_000_000
}

/// The little of `DateTime` the walkers use: the hour, for the night rhythm.
#[derive(Clone, Copy, Default)]
pub struct DateTime {
    secs: i64,
    local: bool,
}
impl DateTime {
    pub fn now() -> Self {
        DateTime {
            secs: unix_now() + local_offset_seconds(),
            local: true,
        }
    }
    pub fn utc_now() -> Self {
        DateTime {
            secs: unix_now(),
            local: false,
        }
    }
    pub fn hour(&self) -> i32 {
        ((self.secs.rem_euclid(86400)) / 3600) as i32
    }
    pub fn minute(&self) -> i32 {
        ((self.secs.rem_euclid(3600)) / 60) as i32
    }
    pub fn ticks(&self) -> i64 {
        self.secs * 10_000_000 + 621_355_968_000_000_000
    }
}
// ---- The local clock (a port) ------------------------------------------------------------

/// Where local time comes from. The domain cannot ask the OS, so the composition root
/// installs an implementation once at start-up; until then local time is UTC.
pub trait LocalClock: Send + Sync {
    /// Seconds to add to UTC for local time.
    fn utc_offset_seconds(&self) -> i64;
}
static LOCAL_CLOCK: OnceLock<Box<dyn LocalClock>> = OnceLock::new();

/// Installs the local clock; only the first call counts.
pub fn install_local_clock(clock: Box<dyn LocalClock>) {
    let _ = LOCAL_CLOCK.set(clock);
}
fn local_offset_seconds() -> i64 {
    LOCAL_CLOCK.get().map_or(0, |c| c.utc_offset_seconds())
}

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64)
}

// ---- System.Drawing ---------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}
impl Point {
    pub const fn new(x: i32, y: i32) -> Self {
        Point { x, y }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub struct Rectangle {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}
impl Rectangle {
    pub const fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Rectangle {
            x,
            y,
            width,
            height,
        }
    }
    pub fn from_ltrb(l: i32, t: i32, r: i32, b: i32) -> Self {
        Rectangle::new(l, t, r - l, b - t)
    }
    #[inline]
    pub fn left(self) -> i32 {
        self.x
    }
    #[inline]
    pub fn top(self) -> i32 {
        self.y
    }
    #[inline]
    pub fn right(self) -> i32 {
        self.x + self.width
    }
    #[inline]
    pub fn bottom(self) -> i32 {
        self.y + self.height
    }
    pub fn is_empty(self) -> bool {
        self.x == 0 && self.y == 0 && self.width == 0 && self.height == 0
    }
    pub fn contains(self, p: Point) -> bool {
        p.x >= self.x && p.x < self.right() && p.y >= self.y && p.y < self.bottom()
    }
    pub fn contains_2(self, r: Rectangle) -> bool {
        self.contains_rect(r)
    }
    pub fn contains_rect(self, r: Rectangle) -> bool {
        self.x <= r.x && r.right() <= self.right() && self.y <= r.y && r.bottom() <= self.bottom()
    }
}

// ---- System.Random -----------------------------------------------------------------

/// .NET's subtractive generator, value for value: a seeded run here matches a
/// seeded run of the C# original, which is how the port is checked.
#[derive(Clone)]
pub struct RandomData {
    seeds: [i32; 56],
    inext: usize,
    inextp: usize,
}
impl Default for RandomData {
    fn default() -> Self {
        RandomData {
            seeds: [0; 56],
            inext: 0,
            inextp: 21,
        }
    }
}
handle!(Random, RandomData);
impl Random {
    pub fn new() -> Random {
        let t = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(1, |d| d.subsec_nanos() as i32);
        Random::with_seed(t)
    }
    pub fn new_2(seed: i32) -> Random {
        Self::with_seed(seed)
    }
    pub fn with_seed(seed: i32) -> Random {
        let mut d = RandomData::default();
        let subtraction = if seed == i32::MIN {
            i32::MAX
        } else {
            seed.abs()
        };
        let mut mj = 161803398i32.wrapping_sub(subtraction);
        d.seeds[55] = mj;
        let mut mk = 1i32;
        for i in 1..55 {
            let ii = (21 * i) % 55;
            d.seeds[ii] = mk;
            mk = mj.wrapping_sub(mk);
            if mk < 0 {
                mk = mk.wrapping_add(i32::MAX);
            }
            mj = d.seeds[ii];
        }
        for _ in 1..5 {
            for i in 1..56 {
                d.seeds[i] = d.seeds[i].wrapping_sub(d.seeds[1 + (i + 30) % 55]);
                if d.seeds[i] < 0 {
                    d.seeds[i] = d.seeds[i].wrapping_add(i32::MAX);
                }
            }
        }
        d.inext = 0;
        d.inextp = 21;
        Random::from_data(d)
    }
    fn internal_sample(mut self) -> i32 {
        let mut a = self.inext;
        let mut b = self.inextp;
        a += 1;
        if a >= 56 {
            a = 1;
        }
        b += 1;
        if b >= 56 {
            b = 1;
        }
        let mut r = self.seeds[a].wrapping_sub(self.seeds[b]);
        if r == i32::MAX {
            r -= 1;
        }
        if r < 0 {
            r = r.wrapping_add(i32::MAX);
        }
        self.seeds[a] = r;
        self.inext = a;
        self.inextp = b;
        r
    }
    #[inline]
    pub fn next_double(self) -> f64 {
        self.internal_sample() as f64 * (1.0 / i32::MAX as f64)
    }
    #[inline]
    pub fn next(self, n: i32) -> i32 {
        (self.next_double() * n as f64) as i32
    }
    pub fn next_2(self, a: i32, b: i32) -> i32 {
        a + (self.next_double() * (b - a) as f64) as i32
    }
}

// ---- StringBuilder ---------------------------------------------------------------------

#[derive(Clone, Default)]
pub struct StringBuilderData {
    pub s: String,
}
handle!(StringBuilder, StringBuilderData);
impl StringBuilder {
    pub fn new() -> StringBuilder {
        StringBuilder::from_data(StringBuilderData::default())
    }
    pub fn append_str(mut self, x: &str) -> StringBuilder {
        self.s.push_str(x);
        self
    }
    pub fn append_disp<D: Display>(mut self, x: D) -> StringBuilder {
        use std::fmt::Write;
        let _ = write!(self.s, "{}", x);
        self
    }
    pub fn length(self) -> i32 {
        self.s.chars().count() as i32
    }
    pub fn set_length(mut self, n: i32) {
        let t: String = self.s.chars().take(n as usize).collect();
        self.s = t;
    }
    /// Takes the text and frees the builder; a builder only ever builds one string.
    pub fn to_string(self) -> String {
        let t = self.s.clone();
        self.free();
        t
    }
}

// ---- The terrain ----------------------------------------------------------------------------

/// What a walker needs to know about the terrain. Keeps the behaviour free of any OS.
pub trait IWorldApi {
    fn left(&self) -> i32;
    fn right(&self) -> i32;
    fn monitor_edges(&self) -> Arr<i32>;
    fn monitor_bounds_at(&self, p: Point) -> Rectangle;
    fn ground_at(&self, x: f64) -> f64;
    fn scale_at(&self, x: f64) -> i32;
}
#[derive(Clone, Copy, Default)]
pub struct IWorld(Option<&'static dyn IWorldApi>);
unsafe impl Send for IWorld {}
unsafe impl Sync for IWorld {}
impl IWorld {
    /// Worlds live as long as the program.
    pub fn of<W: IWorldApi + 'static>(w: W) -> Self {
        IWorld(Some(Box::leak(Box::new(w))))
    }
    pub fn left(&self) -> i32 {
        self.0.unwrap().left()
    }
    pub fn right(&self) -> i32 {
        self.0.unwrap().right()
    }
    pub fn monitor_edges(&self) -> Arr<i32> {
        self.0.unwrap().monitor_edges()
    }
    pub fn monitor_bounds_at(&self, p: Point) -> Rectangle {
        self.0.unwrap().monitor_bounds_at(p)
    }
    pub fn ground_at(&self, x: f64) -> f64 {
        self.0.unwrap().ground_at(x)
    }
    pub fn scale_at(&self, x: f64) -> i32 {
        self.0.unwrap().scale_at(x)
    }
}
