use crate::math::{ToF32, ToU32};
use atomic_float::AtomicF32;
use std::sync::atomic::{self, AtomicU32};

static RENDER_STEP: AtomicF32 = AtomicF32::new(1.0 / 60.0);
static UPDATE_STEP: AtomicF32 = AtomicF32::new(1.0 / 60.0);
static FPS: AtomicU32 = AtomicU32::new(0);
static UPS: AtomicU32 = AtomicU32::new(0);

pub fn set_target_fps<T>(target: T)
where
    T: ToF32,
{
    RENDER_STEP.store(1.0 / target.to_f32(), atomic::Ordering::Relaxed);
}

pub fn set_target_ups<T>(target: T)
where
    T: ToF32,
{
    UPDATE_STEP.store(1.0 / target.to_f32(), atomic::Ordering::Relaxed);
}

pub fn render_step() -> f32 {
    RENDER_STEP.load(atomic::Ordering::Relaxed)
}

pub fn update_step() -> f32 {
    UPDATE_STEP.load(atomic::Ordering::Relaxed)
}

pub(crate) fn set_fps<T>(fps: T)
where
    T: ToU32,
{
    FPS.store(fps.to_u32(), atomic::Ordering::Relaxed);
}

pub fn fps() -> u32 {
    FPS.load(atomic::Ordering::Relaxed)
}

pub(crate) fn set_ups<T>(ups: T)
where
    T: ToU32,
{
    UPS.store(ups.to_u32(), atomic::Ordering::Relaxed);
}

pub fn ups() -> u32 {
    UPS.load(atomic::Ordering::Relaxed)
}

// static mut RENDER_STEP: f32 = 1.0 / 60.0;
// static mut UPDATE_STEP: f32 = 1.0 / 60.0;
// static mut FPS: u32 = 0;
// static mut UPS: u32 = 0;

// pub fn set_target_fps<T>(target: T)
// where
//     T: ToF32,
// {
//     unsafe {
//         RENDER_STEP = 1.0 / target.to_f32();
//     }
// }

// pub fn set_target_ups<T>(target: T)
// where
//     T: ToF32,
// {
//     unsafe {
//         UPDATE_STEP = 1.0 / target.to_f32();
//     }
// }

// pub fn render_step() -> f32 {
//     unsafe { RENDER_STEP }
// }

// pub fn update_step() -> f32 {
//     unsafe { UPDATE_STEP }
// }

// pub(crate) fn set_fps<T>(fps: T)
// where
//     T: ToU32,
// {
//     unsafe {
//         FPS = fps.to_u32();
//     }
// }

// pub fn fps() -> u32 {
//     unsafe { FPS }
// }

// pub(crate) fn set_ups<T>(ups: T)
// where
//     T: ToU32,
// {
//     unsafe {
//         UPS = ups.to_u32();
//     }
// }

// pub fn ups() -> u32 {
//     unsafe { UPS }
// }
