use nostd::alloc::vec::Vec;
use sdl3::monitor::Monitor as SdlMonitor;
use sdl3::monitor::MonitorId;

#[derive(Debug, Clone, Copy)]
pub struct Monitor(SdlMonitor);

impl From<SdlMonitor> for Monitor {
    fn from(value: SdlMonitor) -> Self {
        Self(value)
    }
}

impl Monitor {
    #[inline]
    pub fn id(&self) -> MonitorId {
        self.0.id
    }

    #[inline]
    pub fn position(&self) -> math::Vector2<i32> {
        self.0.position
    }

    #[inline]
    pub fn size(&self) -> math::Size<u32> {
        self.0.size
    }

    #[inline]
    pub fn pixel_density(&self) -> f32 {
        self.0.pixel_density
    }

    #[inline]
    pub fn refresh_rate(&self) -> f32 {
        self.0.refresh_rate
    }
}

pub struct Monitors {
    all: Vec<Monitor>,
    primary: Option<Monitor>,
}

impl Monitors {
    pub(crate) fn new() -> Self {
        let mut this = Self {
            all: Vec::new(),
            primary: None,
        };

        this.refresh();
        this
    }

    pub(crate) fn refresh(&mut self) {
        self.all = SdlMonitor::all().into_iter().map(Monitor::from).collect();
        self.primary = SdlMonitor::primary().map(Monitor::from);
    }

    #[inline]
    pub fn all(&self) -> &[Monitor] {
        &self.all
    }

    #[inline]
    pub fn primary(&self) -> Option<Monitor> {
        self.primary
    }
}
