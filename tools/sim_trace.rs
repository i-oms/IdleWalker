//! Seeded behaviour trace, twin of tools/compare/SimTrace.cs: same world, seed and
//! steps. A diff of the two outputs is empty exactly when the simulation matches.
use idlewalker_domain::prelude::*;
use idlewalker_domain::rt::IWorldApi;

struct TraceWorld(Arr<i32>);
impl IWorldApi for TraceWorld {
    fn left(&self) -> i32 {
        0
    }
    fn right(&self) -> i32 {
        3200
    }
    fn monitor_edges(&self) -> Arr<i32> {
        self.0.clone()
    }
    fn monitor_bounds_at(&self, p: Point) -> Rectangle {
        if p.x < 1920 {
            Rectangle::new(0, 0, 1920, 1080)
        } else {
            Rectangle::new(1920, 0, 1280, 800)
        }
    }
    fn ground_at(&self, x: f64) -> f64 {
        if x < 1920.0 {
            1040.0
        } else {
            760.0
        }
    }
    fn scale_at(&self, x: f64) -> i32 {
        if x < 1920.0 {
            3
        } else {
            4
        }
    }
}

fn mix(h: u64, d: f64) -> u64 {
    (h ^ d.to_bits()).wrapping_mul(1099511628211)
}

fn main() {
    // The walkers' night rhythm follows the local clock, as in the C# original.
    idlewalker_domain::rt::install_local_clock(Box::new(idlewalker_platform::SystemClock));
    let world = IWorld::of(TraceWorld(Arr::from_vec(vec![0, 1920, 1920, 3200])));
    let troupe = Troupe::new(world, Random::with_seed(1234));
    troupe.ensure_cast();
    troupe.set_minion_mask(Troupe::all_minions_mask());
    troupe.set_bird_mask(Troupe::all_birds_mask());
    troupe.set_minions_enabled(true);
    troupe.set_birds_enabled(true);
    let dt = 1.0 / 60.0;
    let mut h: u64 = 14695981039346656037;
    for i in 0..60 * 600 {
        let cursor = Point::new(600 + (i / 3000) * 300, 300);
        let disturbed = i % 18000 == 17999;
        troupe.step(dt, cursor, i > 10 && !disturbed, disturbed);
        for w in troupe.walkers().items() {
            let p = w.current_pose();
            h = mix(h, w.centre_x());
            h = mix(h, w.feet_y());
            h = mix(h, p.yaw);
            h = mix(h, p.arm_near);
            h = mix(h, p.face.mouth_curve);
        }
        if i % 600 == 0 {
            let mut s = format!("{} {:016X}", i, h);
            for w in troupe.walkers().items() {
                s += &format!(
                    " {}={}@{}",
                    w.personality.name,
                    w.activity(),
                    w.centre_x() as i32
                );
            }
            println!("{}", s);
        }
    }
    for w in troupe.walkers().items() {
        println!(
            "{} exp={} tastes={} friends={}",
            w.personality.name,
            w.experience(),
            w.learned_entries(),
            w.friendships()
        );
    }
}
