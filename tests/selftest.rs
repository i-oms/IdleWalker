//! The C# original's tools/SelfTest.cs: the cast learns headless, and memories survive
//! a save and load, including files from older versions.
use idlewalker_application::Brains;
use idlewalker_domain::prelude::*;
use idlewalker_domain::rt::IWorldApi;
use idlewalker_platform::FileBrainStore;
use std::collections::HashMap;
use std::sync::Arc;

struct FakeWorld(Arr<i32>);
impl IWorldApi for FakeWorld {
    fn left(&self) -> i32 {
        0
    }
    fn right(&self) -> i32 {
        1920
    }
    fn monitor_edges(&self) -> Arr<i32> {
        self.0.clone()
    }
    fn monitor_bounds_at(&self, _p: Point) -> Rectangle {
        Rectangle::new(0, 0, 1920, 1040)
    }
    fn ground_at(&self, _x: f64) -> f64 {
        1000.0
    }
    fn scale_at(&self, _x: f64) -> i32 {
        3
    }
}

fn world() -> IWorld {
    // The walkers' night rhythm follows the local clock, as the program's does.
    idlewalker_domain::rt::install_local_clock(Box::new(idlewalker_platform::SystemClock));
    IWorld::of(FakeWorld(Arr::from_vec(vec![0, 1920])))
}

/// Two minutes of five Minions.
fn lived_troupe() -> Troupe {
    let troupe = Troupe::new(world(), Random::with_seed(1234));
    troupe.ensure_cast();
    troupe.set_minions_enabled(true);
    troupe.set_count(5);
    for i in 0..60 * 120 {
        troupe.step(1.0 / 60.0, Point::new(960, 200), i > 10, false);
    }
    troupe
}

fn sum(t: Troupe, f: impl Fn(Walker) -> i32) -> i32 {
    t.walkers().items().map(f).sum()
}

#[test]
fn the_cast_learns_headless() {
    let t = lived_troupe();
    assert!(
        t.walkers().items().any(|w| w.is_visible()),
        "walkers appear headless"
    );
    assert!(sum(t, |w| w.experience()) > 0, "experience tracking");
    assert!(sum(t, |w| w.learned_entries()) > 0, "tabular values learn");
    let net: f64 = t.walkers().items().map(|w| w.mlp_weight_norm()).sum();
    assert!(net > 0.01, "the network moved from its initial state");
    for w in t.walkers().items().filter(|w| w.is_visible()) {
        assert!(!w.activity().is_empty(), "activities are never blank");
    }
}

#[test]
fn a_mind_survives_pack_and_unpack() {
    let t = lived_troupe();
    let first = t.walkers().items().find(|w| w.is_visible()).unwrap();
    let pack = first.pack_mind();
    for key in ["exp=", "rew=", "mw1=", "mshape="] {
        assert!(pack.contains(key), "pack contains {}", key);
    }
    let fields: Dict<String, String> = Dict::new();
    for line in pack.lines() {
        if let Some((k, v)) = line.trim().split_once('=') {
            fields.set(k.to_string(), v.to_string());
        }
    }
    let other = Troupe::new(world(), Random::with_seed(999));
    other.ensure_cast();
    let by_name: Dict<String, Walker> = Dict::new();
    for w in other.walkers().items() {
        by_name.set(w.personality.name.clone(), w);
    }
    let target = by_name.get(&first.personality.name);
    assert_eq!(
        target.experience(),
        0,
        "a fresh walker starts at 0 decisions"
    );
    target.unpack_mind(fields, by_name.clone());
    assert_eq!(target.experience(), first.experience());
    assert!((target.avg_reward() - first.avg_reward()).abs() < 1e-4);
    assert!((target.mlp_weight_norm() - first.mlp_weight_norm()).abs() < 1e-4);

    // Old files have few fields; they must still be accepted.
    let v1: Dict<String, String> = Dict::new();
    v1.set("opt".to_string(), "0.1".to_string());
    v1.set("last".to_string(), "-1,0".to_string());
    target.forget_mind();
    target.unpack_mind(v1, by_name);
    assert!(target.mlp_weight_norm() >= 0.0);

    first.forget_mind();
    assert_eq!(first.experience(), 0);
    assert_eq!(first.mlp_weight_norm(), 0.0, "forget clears the network");
}

#[test]
fn brains_survive_on_disk() {
    let dir = std::env::temp_dir().join(format!("idlewalker-selftest-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("brains.txt");
    let store = Brains::new(Arc::new(FileBrainStore::at(path.clone())));

    let t = lived_troupe();
    store.save(t);
    assert!(path.exists(), "saved");
    assert!(!dir.join("brains.txt.tmp").exists(), "no tmp left behind");
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("# IdleWalker brains v4"), "v4 header");
    assert!(
        text.contains("\nvs=") && !text.contains("\nv="),
        "v4 table written sparse"
    );

    let loaded = Troupe::new(world(), Random::with_seed(555));
    store.load(loaded);
    assert_eq!(
        sum(loaded, |w| w.experience()),
        sum(t, |w| w.experience()),
        "decisions preserved"
    );
    assert_eq!(
        sum(loaded, |w| w.learned_entries()),
        sum(t, |w| w.learned_entries()),
        "every learned cell preserved"
    );

    // A v3 file (eight situations, dense table) still loads, spread over the fine situations.
    let cols = PERSONALITY_ROSTER[2].habits.len() + 3; // Bob
    let dense: Vec<&str> = (0..8 * cols)
        .map(|i| if i % cols == 0 { "0.5" } else { "0" })
        .collect();
    std::fs::write(
        &path,
        format!(
            "# IdleWalker brains v3\n[Bob]\nvrows=8\nvcols={}\nv={}\nexp=42\n",
            cols,
            dense.join(",")
        ),
    )
    .unwrap();
    let old = Troupe::new(world(), Random::with_seed(999));
    store.load(old);
    let bob = old
        .walkers()
        .items()
        .find(|w| w.personality.name == "Bob")
        .unwrap();
    assert_eq!(bob.experience(), 42, "v3 file still loads");
    assert_eq!(
        bob.learned_entries(),
        81,
        "v3 table spread over the 81 fine situations"
    );

    // A file from when the cast was larger still loads: a walker no longer in the cast
    // is skipped, and so is a friendship with one, without touching anyone else.
    std::fs::write(
        &path,
        "# IdleWalker brains v4\n[Retired]\nexp=7\n[Bob]\nexp=42\nfond=Retired,0.5;Kevin,0.25\n",
    )
    .unwrap();
    let older = Troupe::new(world(), Random::with_seed(888));
    store.load(older);
    let bob = older
        .walkers()
        .items()
        .find(|w| w.personality.name == "Bob")
        .unwrap();
    assert_eq!(
        bob.experience(),
        42,
        "the walkers still here keep their memories"
    );
    assert_eq!(
        bob.friendships(),
        1,
        "only the friendship with a walker still here"
    );
    assert_eq!(
        sum(older, |w| w.experience()),
        42,
        "nobody picks up the retired walker's memories"
    );

    // A foreign file must not crash and must not load.
    std::fs::write(&path, "# foreign file\n[Bob]\nexp=9999\n").unwrap();
    let foreign = Troupe::new(world(), Random::with_seed(777));
    foreign.ensure_cast();
    store.load(foreign);
    assert_eq!(
        sum(foreign, |w| w.experience()),
        0,
        "unknown header ignored"
    );

    let _ = std::fs::remove_dir_all(&dir);
    let _ = HashMap::<i32, i32>::new();
}

#[test]
fn birds_hop_and_rouse() {
    let t = Troupe::new(world(), Random::with_seed(1234));
    t.ensure_cast();
    t.set_minions_enabled(false);
    t.set_birds_enabled(true);
    t.set_bird_mask(Troupe::all_birds_mask());
    let (mut hops, mut rouses) = (0, 0);
    for i in 0..60 * 300 {
        t.step(1.0 / 60.0, Point::new(960, 200), i > 10, false);
        for w in t.walkers().items().filter(|w| w.is_visible()) {
            assert_eq!(
                w.personality.look.figure,
                Figure::Bird,
                "only birds are out"
            );
            let pose = w.current_pose();
            assert!(
                pose.body_dy.is_finite() && pose.stretch_y > 0.0 && pose.stretch_x > 0.0,
                "{} in {:?}: dy={} sx={} sy={}",
                w.personality.name,
                w.mode,
                pose.body_dy,
                pose.stretch_x,
                pose.stretch_y
            );
            if w.mode == Mode::Walk && pose.body_dy < 0.0 {
                hops += 1;
            }
            if (w.mode == Mode::Rest || w.mode == Mode::Look) && pose.puff_up > 0.0 {
                rouses += 1;
            }
        }
    }
    assert!(hops > 0, "a walking bird leaves the ground");
    assert!(rouses > 0, "a resting bird fluffs itself up now and then");
}

#[test]
fn seeded_random_matches_dotnet() {
    // Values from .NET's System.Random(1234).
    let r = Random::with_seed(1234);
    let v: Vec<i32> = (0..3).map(|_| r.next(1000)).collect();
    assert_eq!(v, vec![399, 895, 319]);
}
