/// Delta time smothing factor
/// Gives the user a more smoothed-out delta time
/// to prevent weird spikes
pub const DELTA_SMOOTHING: f32 = 0.2;

/// Base vertex capacity for buffers in immediate rendering mode
pub const IMMEDIATE_VERTEX_BASE_CAPACITY: usize = 4096;

/// Base index capacity for buffers in immediate rendering mode
pub const IMMEDIATE_INDEX_BASE_CAPACITY: usize = IMMEDIATE_VERTEX_BASE_CAPACITY * 2;

/// Base capacity for mesh instance buffer, it means up to 1024 instances of the same mesh
/// can be stored until resizing
pub const MESH_INSTANCE_BASE_CAPACITY: usize = 1024;

/// Base capacity for text instance buffer, it means up to 1024 instances of the same text
/// can be stored until resizing
pub const TEXT_INSTANCE_BASE_CAPACITY: usize = 1024;

/// Base size of the texture atlas, can be resized
pub const TEXTURE_ATLAS_BASE_SIZE: (u32, u32) = (1024, 1024);
