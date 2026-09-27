use core::ops::Deref;
use core::time::Duration;

use crate::commands::InputCommand;
use crate::commands::Outbox;
use crate::input::Input;
use crate::input::PadView;

pub struct InputHandle<'a> {
    pub(crate) data: &'a Input,
    pub(crate) outbox: &'a mut Outbox<InputCommand>,
}

impl Deref for InputHandle<'_> {
    type Target = Input;

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl InputHandle<'_> {
    pub fn pad_mut(&mut self, slot: usize) -> Option<PadHandle<'_>> {
        let view = self.data.pad(slot)?;

        Some(PadHandle {
            view,
            slot,
            outbox: self.outbox,
        })
    }

    pub fn rumble(&mut self, slot: usize, intensity: f32, duration: Duration) {
        if let Some(mut pad) = self.pad_mut(slot) {
            pad.rumble(intensity, duration);
        }
    }

    pub fn rumble_motors(&mut self, slot: usize, low: f32, high: f32, duration: Duration) {
        if let Some(mut pad) = self.pad_mut(slot) {
            pad.rumble_motors(low, high, duration);
        }
    }

    pub fn rumble_triggers(&mut self, slot: usize, left: f32, right: f32, duration: Duration) {
        if let Some(mut pad) = self.pad_mut(slot) {
            pad.rumble_triggers(left, right, duration);
        }
    }

    pub fn stop_rumble(&mut self, slot: usize) {
        if let Some(mut pad) = self.pad_mut(slot) {
            pad.stop_rumble();
        }
    }
}

pub struct PadHandle<'a> {
    view: PadView<'a>,
    slot: usize,
    outbox: &'a mut Outbox<InputCommand>,
}

impl<'a> Deref for PadHandle<'a> {
    type Target = PadView<'a>;

    fn deref(&self) -> &Self::Target {
        &self.view
    }
}

impl PadHandle<'_> {
    pub fn rumble(&mut self, intensity: f32, duration: Duration) {
        self.rumble_motors(intensity, intensity, duration);
    }

    pub fn rumble_motors(&mut self, low: f32, high: f32, duration: Duration) {
        self.outbox.push(InputCommand::Rumble {
            slot: self.slot,
            low,
            high,
            duration,
        });
    }

    pub fn rumble_triggers(&mut self, left: f32, right: f32, duration: Duration) {
        self.outbox.push(InputCommand::RumbleTriggers {
            slot: self.slot,
            left,
            right,
            duration,
        });
    }

    pub fn stop_rumble(&mut self) {
        self.rumble_motors(0.0, 0.0, Duration::ZERO);
        self.rumble_triggers(0.0, 0.0, Duration::ZERO);
    }
}
