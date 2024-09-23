pub(crate) static mut FPS: u32 = 0;
pub(crate) static mut UPS: u32 = 0;
pub(crate) static mut FRAME_STEP: f32 = 1.0 / 60.0;
pub(crate) static mut UPDATE_STEP: f32 = 1.0 / 60.0;

pub fn fps() -> u32 {
    unsafe { FPS }
}

pub fn ups() -> u32 {
    unsafe { UPS }
}

pub(crate) fn frame_step() -> f32 {
    unsafe { FRAME_STEP }
}

pub(crate) fn update_step() -> f32 {
    unsafe { UPDATE_STEP }
}

pub fn set_target_fps(fps: u32) {
    unsafe {
        FRAME_STEP = 1.0 / fps as f32;
    }
}

pub fn set_target_ups(ups: u32) {
    unsafe {
        UPDATE_STEP = 1.0 / ups as f32;
    }
}
