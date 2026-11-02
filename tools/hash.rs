//! Prints an FNV-1a hash of the painted canvas for a fixed grid of poses across
//! all four figures: the same grid as the C# original's tools/Hash.cs, so a diff
//! of the two outputs is empty exactly when the painters are pixel-identical.
use idlewalker_rendering::prelude::*;

fn fnv(data: &Arr<i32>, n: i32) -> u64 {
    let mut h: u64 = 14695981039346656037;
    for i in 0..n {
        h ^= data[i] as u32 as u64;
        h = h.wrapping_mul(1099511628211);
    }
    h
}

fn main() {
    let (roster, birds) = (&*PERSONALITY_ROSTER, &*PERSONALITY_BIRD_ROSTER);
    let looks = [
        roster[0].look,
        roster[1].look,
        roster[4].look,
        roster[7].look,
        PERSONALITY_BEAN.look,
        PERSONALITY_PANTHER.look,
        birds[0].look,
        birds[3].look,
        birds[7].look,
    ];
    let anatomies = [
        *ANATOMY_MINION,
        *ANATOMY_MINION,
        *ANATOMY_MINION,
        *ANATOMY_MINION,
        *ANATOMY_BEAN,
        *ANATOMY_PANTHER,
        *ANATOMY_BIRD,
        *ANATOMY_BIRD,
        *ANATOMY_BIRD,
    ];
    let yaws = [-45.0, 0.0, 30.0, 58.0, 90.0, 150.0];
    let phases = [0.0, 0.3, 0.7];
    let faces = [
        *EXPRESSION_NEUTRAL,
        *EXPRESSION_HAPPY,
        *EXPRESSION_SCARED,
        *EXPRESSION_LAUGH,
        *EXPRESSION_SLEEPY,
    ];
    let dresses = [0x2B59B8, 0xC0392B];

    let surface = Surface::new();
    let mut count = 0;
    for l in 0..looks.len() {
        let scale = 3;
        let (width, height) = (40 * scale, anatomies[l].canvas_rows * scale);
        for yi in 0..yaws.len() {
            for pi in 0..phases.len() {
                for fi in 0..faces.len() {
                    let mut pose = Pose::standing();
                    pose.face = faces[fi];
                    pose.yaw = yaws[yi];
                    pose.walking = pi > 0;
                    pose.walk_phase = phases[pi];
                    pose.gaze_x = 0.25 * yi as f64 - 0.5;
                    pose.gaze_y = 0.1 * fi as f64;
                    pose.lean = 0.1 * pi as f64;
                    pose.body_dy = 0.15 * pi as f64;
                    if fi == 1 {
                        pose.arm_near = 120.0;
                        pose.fore_near = 30.0;
                        pose.point_near = true;
                    }
                    if fi == 2 {
                        pose.arm_near = 70.0;
                        pose.arm_far = -70.0;
                        pose.bubble = true;
                        pose.sweat = 0.4;
                    }
                    if fi == 3 {
                        pose.note = true;
                        pose.decor_phase = 0.35;
                        pose.teddy = true;
                        pose.teddy_wave = 0.5;
                    }
                    if fi == 4 {
                        pose.zzz = true;
                        pose.decor_phase = 0.6;
                        pose.head_dip = 0.3;
                        pose.head_turn = 20.0;
                    }
                    if yi == 5 {
                        pose.leg_tuck = 0.6;
                        pose.stretch_x = 0.95;
                        pose.stretch_y = 1.06;
                        pose.heart = true;
                        pose.decor_phase = 0.5;
                    }
                    if yi == 2 && pi == 1 {
                        pose.wheel = true;
                        pose.wheel_turn = 0.4;
                        pose.egg = true;
                        pose.puff_up = 0.5;
                        pose.tiptoe = 0.5;
                    }
                    if yi == 4 && pi == 2 {
                        pose.camera = true;
                        pose.flash = fi == 0;
                        pose.blast_flash = fi == 1;
                    }
                    let face_left = (yi + pi) % 2 == 1;
                    let dress = dresses[(yi + fi) % 2];

                    surface.reset(width, height);
                    surface.shadow(
                        width as f64 / 2.0,
                        height as f64 - 1.0,
                        anatomies[l].shadow_radius * scale as f64,
                        1.4 * scale as f64,
                        0.4,
                    );
                    surface.puff(width / 2 - 8, height - 2, 4, 120);
                    FrameRenderer::draw_pose(
                        surface,
                        pose,
                        looks[l],
                        dress,
                        face_left,
                        width as f64 / 2.0 + 0.3 * pi as f64,
                        height as f64 - 2.0,
                        scale as f64,
                    );
                    let h = fnv(&surface.layer(), surface.width() * surface.height());
                    let p = fnv(&surface.pixels, surface.width() * surface.height());
                    let b = surface.bounds();
                    println!("{} look={} yaw={} ph={} face={} L={:016X} P={:016X} b={{X={},Y={},Width={},Height={}}}",
                        count, l, yaws[yi], phases[pi], fi, h, p, b.x, b.y, b.width, b.height);
                    count += 1;
                }
            }
        }
    }
}
