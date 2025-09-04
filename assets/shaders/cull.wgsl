@group(0) @binding(0) var<storage, read> input_instances: array<u32>;
@group(0) @binding(1) var<storage, read_write> output_instances: array<u32>;
@group(0) @binding(2) var<storage, read_write> indirect_buffer: array<atomic<u32>>;
@group(0) @binding(3) var<uniform> frustum_planes: array<vec4<f32>, 6>;
@group(0) @binding(4) var<uniform> instance_count: u32;

// Generic instance data - we'll interpret the raw bytes based on type
fn get_vec2_at_offset(instance_data: array<u32, 8>, offset: u32) -> vec2<f32> {
    return vec2<f32>(
        bitcast<f32>(instance_data[offset]),
        bitcast<f32>(instance_data[offset + 1u])
    );
}

fn get_vec4_at_offset(instance_data: array<u32, 8>, offset: u32) -> vec4<f32> {
    return vec4<f32>(
        bitcast<f32>(instance_data[offset]),
        bitcast<f32>(instance_data[offset + 1u]),
        bitcast<f32>(instance_data[offset + 2u]),
        bitcast<f32>(instance_data[offset + 3u])
    );
}

fn point_vs_frustum(pos: vec2<f32>) -> bool {
    // Simple 2D bounds check for points
    let plane_left = frustum_planes[0];   // x >= 0
    let plane_right = frustum_planes[1];  // x <= width  
    let plane_bottom = frustum_planes[2]; // y <= height
    let plane_top = frustum_planes[3];    // y >= 0

    // Check if point is within screen bounds
    if (pos.x < 0.0 || pos.x > plane_right.w || pos.y < 0.0 || pos.y > plane_bottom.w) {
        return false;
    }
    
    return true;
}

fn rect_vs_frustum(center: vec2<f32>, size: vec2<f32>) -> bool {
    let half_size = size * 0.5;
    let min_bounds = center - half_size;
    let max_bounds = center + half_size;
    
    let plane_left = frustum_planes[0];   // x >= 0
    let plane_right = frustum_planes[1];  // x <= width  
    let plane_bottom = frustum_planes[2]; // y <= height
    let plane_top = frustum_planes[3];    // y >= 0

    // AABB vs AABB test
    if (max_bounds.x < 0.0 || min_bounds.x > plane_right.w ||
        max_bounds.y < 0.0 || min_bounds.y > plane_bottom.w) {
        return false;
    }
    
    return true;
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= instance_count) {
        return;
    }

    // Read raw instance data (assuming each instance is 8 u32s = 32 bytes)
    let base_index = index * 8u;
    var instance_data: array<u32, 8>;
    
    for (var i = 0u; i < 8u; i = i + 1u) {
        instance_data[i] = input_instances[base_index + i];
    }

    // For both Pixel and Rect, position is at offset 0
    let position = get_vec2_at_offset(instance_data, 0u);
    
    var is_visible = false;
    
    // Check if this looks like a Pixel (single point) or Rect
    // This is a heuristic - you might need to pass instance type info
    let potential_size = get_vec2_at_offset(instance_data, 2u);
    
    if (potential_size.x > 0.0 && potential_size.y > 0.0 && potential_size.x < 10000.0 && potential_size.y < 10000.0) {
        // Looks like a Rect with valid size
        is_visible = rect_vs_frustum(position, potential_size);
    } else {
        // Treat as point/pixel
        is_visible = point_vs_frustum(position);
    }

    if (is_visible) {
        let output_index = atomicAdd(&indirect_buffer[1], 1u);
        let output_base = output_index * 8u;
        
        // Copy the entire instance data
        for (var i = 0u; i < 8u; i = i + 1u) {
            output_instances[output_base + i] = instance_data[i];
        }
    }
}