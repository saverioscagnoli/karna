use std::cell::OnceCell;

use logging::debug;
use sdl3::gpu::Device;
use sdl3::gpu::Sampler;
use sdl3::gpu::SamplerAddressMode;
use sdl3::gpu::SamplerCreateInfo;
use sdl3::gpu::SamplerMipmapMode;

/// How a texture is filtered when it is sampled at a size other than its own.
///
/// Each variant maps to exactly one device sampler, shared by every texture
/// that asks for it. Adding a variant here is the only thing needed to make a
/// new sampler configuration available.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Filter {
    /// Nearest-neighbour. Keeps pixel art crisp and keeps atlased sprites from
    /// bleeding into their neighbours.
    #[default]
    Nearest,
    /// Bilinear. Smooths scaled-up textures at the cost of blurring edges.
    Linear,
}

impl Filter {
    pub const ALL: [Filter; 2] = [Filter::Nearest, Filter::Linear];

    fn create(self, device: &Device) -> Result<Sampler, sdl3::Error> {
        let filter = match self {
            Filter::Nearest => sdl3::gpu::Filter::Nearest,
            Filter::Linear => sdl3::gpu::Filter::Linear,
        };

        device.create_sampler(
            SamplerCreateInfo::new()
                .with_min_filter(filter)
                .with_mag_filter(filter)
                .with_mipmap_mode(SamplerMipmapMode::Nearest)
                .with_address_mode_u(SamplerAddressMode::ClampToEdge)
                .with_address_mode_v(SamplerAddressMode::ClampToEdge)
                .with_address_mode_w(SamplerAddressMode::ClampToEdge),
        )
    }
}

/// One sampler per [`Filter`], created the first time that filter is bound.
///
/// A program that only ever draws pixel art never pays for a linear sampler.
/// Lookups take `&self` so a texture can reach its sampler while the render
/// pass holds the device borrow; [`OnceCell`] hands back a reference that lives
/// as long as the cache itself, which a `RefCell` map could not.
pub struct SamplerCache {
    samplers: [OnceCell<Sampler>; Filter::ALL.len()],
}

impl SamplerCache {
    pub fn new() -> Self {
        Self {
            samplers: [const { OnceCell::new() }; Filter::ALL.len()],
        }
    }

    /// The sampler for `filter`, creating it on first use.
    pub fn get(&self, device: &Device, filter: Filter) -> &Sampler {
        self.samplers[filter as usize].get_or_init(|| {
            debug!("Created {:?} sampler", filter);

            filter.create(device).expect("Failed to create sampler")
        })
    }
}

impl Default for SamplerCache {
    fn default() -> Self {
        Self::new()
    }
}
