// Translated from the C# original by tools/cs2rs; keeps its statement order.
#![allow(non_upper_case_globals)]
use crate::prelude::*;

// One dust speck as plain values: safe to hand to another thread.
#[derive(Clone, Default, Copy)]
pub struct PuffSnapshot {
    pub x: i32,
    pub y: i32,
    pub size: i32,
    pub alpha: i32,
}

impl PuffSnapshot {}

//
// Everything a frame needs, as plain values: geometry the UI thread
// worked out, the pose struct, and the immutable look. No walker is
// touched while painting, so this may cross to the render thread.
//
#[derive(Clone, Default)]
pub struct RenderJob {
    pub stamp: u32,
    pub width: i32,
    pub height: i32,
    pub left: i32,
    pub top: i32,
    pub has_shadow: bool,
    pub shadow_cx: f64,
    pub shadow_gy: f64,
    pub shadow_rx: f64,
    pub shadow_ry: f64,
    pub shadow_alpha: f64,
    pub puffs: Arr<PuffSnapshot>,
    pub pose: Pose,
    pub look: Look,
    pub dress: i32,
    pub face_left: bool,
    pub char_cx: f64,
    pub sole_y: f64,
    pub scale: f64,
}

impl RenderJob {}

//
// Turns one walker's state into pixels, in two halves: `Prepare`
// runs on the UI thread and only does cheap arithmetic (layout and the
// unchanged check - a walker that stands still costs almost nothing),
// while `PaintJob` does the dear raster work from the
// snapshot and may run anywhere. Finished frames are shown by the UI
// thread, which owns the windows.
//
#[derive(Clone, Default)]
pub struct FrameRendererData {
    pub has_last: bool,
    pub last_pose: Pose,
    pub last_left: i32,
    pub last_top: i32,
    pub last_feet: i32,
    pub last_ground: i32,
    pub last_scale: i32,
    pub last_face_left: bool,
    pub last_had_dust: bool,
    pub last_dress: i32,
    pub stamp: u32,
}
rt::handle!(FrameRenderer, FrameRendererData);

impl FrameRenderer {
    fn alloc__() -> FrameRenderer {
        let mut this = FrameRenderer::from_data(FrameRendererData::default());
        this.stamp = 1;
        this
    }
    pub fn new() -> FrameRenderer {
        Self::alloc__()
    }
    pub const CANVAS_COLUMNS: i32 = 40;
    pub const ROWS_BELOW_FEET: i32 = 18;
    // Stamp of the latest snapshot; results with an older one are stale.
    pub fn stamp(mut self) -> u32 {
        return self.stamp;
    }
    // Forget the last frame, so the next one is drawn for sure.
    pub fn invalidate(mut self) {
        self.has_last = false;
        self.stamp += 1;
    }
    //
    // Snapshots the walker; false when the picture would be identical to
    // the last frame and there is nothing to paint.
    //
    pub fn prepare(mut self, mut walker: Walker, job: &mut RenderJob) -> bool {
        // The canvas rests on the ground line, so anything underground is
        // simply cut off. In a high jump it follows the character up.
        (*job) = RenderJob::default();
        let mut scale: i32 = walker.scale();
        let mut anatomy: Anatomy = walker.personality().anatomy();
        let mut width: i32 = (FrameRenderer::CANVAS_COLUMNS * scale);
        let mut height: i32 = (anatomy.canvas_rows * scale);
        let mut ground: i32 = FrameRenderer::round(walker.ground_line());
        let mut top: i32 = (FrameRenderer::round(rt::min_f64(
            walker.ground_line(),
            (walker.feet_y() + ((FrameRenderer::ROWS_BELOW_FEET * scale) as f64)),
        )) - height);
        let mut left: i32 = (FrameRenderer::round(walker.centre_x()) - (width / 2));
        let mut feet: i32 = FrameRenderer::round(walker.feet_y());
        let mut has_dust: bool = (walker.puffs().len() > 0);
        let mut pose: Pose = walker.current_pose();
        let mut dress: i32 = walker.dress;
        let mut face_left: bool = walker.faces_left();
        let mut unchanged: bool = ((((((((((self.has_last && !(has_dust))
            && !(self.last_had_dust))
            && (left == self.last_left))
            && (top == self.last_top))
            && (feet == self.last_feet))
            && (ground == self.last_ground))
            && (scale == self.last_scale))
            && (face_left == self.last_face_left))
            && (dress == self.last_dress))
            && pose.same_as(self.last_pose));
        if unchanged {
            return false;
        }
        self.has_last = true;
        self.last_pose = pose;
        self.last_left = left;
        self.last_top = top;
        self.last_feet = feet;
        self.last_ground = ground;
        self.last_scale = scale;
        self.last_face_left = face_left;
        self.last_dress = dress;
        self.last_had_dust = has_dust;
        (*job).stamp = self.stamp;
        (*job).width = width;
        (*job).height = height;
        (*job).left = left;
        (*job).top = top;
        // The shadow stays on the ground and thins out as it jumps.
        let mut lift: f64 = ((walker.ground_line() - walker.feet_y()) / (40.0 * ((scale) as f64)));
        if (lift >= 0.0) && (lift < 1.0) {
            (*job).has_shadow = true;
            (*job).shadow_cx = (walker.centre_x() - ((left) as f64));
            (*job).shadow_gy = (walker.ground_line() - ((top) as f64));
            (*job).shadow_rx =
                (((anatomy.shadow_radius - (2.2 * lift)) * ((scale) as f64)) * pose.stretch_x);
            (*job).shadow_ry = (1.4 * ((scale) as f64));
            (*job).shadow_alpha = (0.42 * (1.0 - lift));
        }
        if has_dust {
            let mut snaps: Arr<PuffSnapshot> = Arr::new(walker.puffs().len());
            {
                let mut k: i32 = 0;
                'l1: while k < snaps.len() {
                    let mut puff: Puff = (*walker.puffs().at(k));
                    let mut age: f64 = puff.age();
                    (*snaps.at(k)).x = (FrameRenderer::round(puff.x) - left);
                    (*snaps.at(k)).y = (FrameRenderer::round(puff.y) - top);
                    (*snaps.at(k)).size = rt::max_i32(
                        1,
                        FrameRenderer::round((((scale) as f64) * (0.7 + (0.7 * age)))),
                    );
                    (*snaps.at(k)).alpha = ((190.0 * (1.0 - age)) as i32);
                    k += 1;
                }
            }
            (*job).puffs = snaps.clone();
        }
        (*job).pose = pose;
        (*job).look = walker.personality().look;
        (*job).dress = dress;
        (*job).face_left = face_left;
        // The soles sit just above the ground line, leaving room for the halo.
        (*job).char_cx = (walker.centre_x() - ((left) as f64));
        (*job).sole_y = ((walker.feet_y() - ((top) as f64)) - 1.0);
        (*job).scale = ((scale) as f64);
        return true;
    }
    // Paints a snapshot into a surface; touches only the two.
    pub fn paint_job(job: &mut RenderJob, mut surface: Surface) {
        surface.reset((*job).width, (*job).height);
        if (*job).has_shadow {
            surface.shadow(
                (*job).shadow_cx,
                (*job).shadow_gy,
                (*job).shadow_rx,
                (*job).shadow_ry,
                (*job).shadow_alpha,
            );
        }
        if !(*job).puffs.is_null() {
            {
                let mut k: i32 = 0;
                'l1: while k < (*job).puffs.len() {
                    surface.puff(
                        (*(*job).puffs.at(k)).x,
                        (*(*job).puffs.at(k)).y,
                        (*(*job).puffs.at(k)).size,
                        (*(*job).puffs.at(k)).alpha,
                    );
                    k += 1;
                }
            }
        }
        FrameRenderer::draw_pose(
            surface,
            (*job).pose,
            (*job).look,
            (*job).dress,
            (*job).face_left,
            (*job).char_cx,
            (*job).sole_y,
            (*job).scale,
        );
    }
    //
    // Draws a pose with its soles resting on soleY, centred on centreX,
    // and commits it to the canvas, with the painter the look calls
    // for. Also used for the About card.
    //
    pub fn draw_pose(
        mut surface: Surface,
        mut pose: Pose,
        mut look: Look,
        mut dress: i32,
        mut face_left: bool,
        mut centre_x: f64,
        mut sole_y: f64,
        mut scale: f64,
    ) {
        if look.figure == Figure::Bean {
            BeanPainter::draw(surface, pose, face_left, centre_x, sole_y, scale);
        } else if look.figure == Figure::Bird {
            BirdPainter::draw(surface, pose, look, face_left, centre_x, sole_y, scale);
        } else if look.figure == Figure::Panther {
            PantherPainter::draw(surface, pose, face_left, centre_x, sole_y, scale);
        } else {
            MinionPainter::draw(
                surface, pose, look, dress, face_left, centre_x, sole_y, scale,
            );
        }
        surface.commit_character(1);
    }
    pub fn round(mut value: f64) -> i32 {
        return (((value + 0.5).floor()) as i32);
    }
}
