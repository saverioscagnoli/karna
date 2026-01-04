use crate::{MeshHandle, Renderer, Transform};
use macros::{Get, Set, track_dirty};
use math::{Vector2, Vector3};

#[derive(Debug, Clone)]
struct MeshEntry {
    handle: MeshHandle,
    local_transform: Transform,
}

#[track_dirty]
#[derive(Default, Debug, Clone)]
#[derive(Get, Set)]
pub struct MeshGroup {
    entries: Vec<MeshEntry>,

    #[get]
    #[get(prop = "position", ty = &Vector3, name = "position")]
    #[get(copied, prop = "position", name = "position_2d", pre = truncate, ty = Vector2)]
    #[get(copied, prop = "position.x", ty = f32, name = "position_x")]
    #[get(copied, prop = "position.y", ty = f32, name = "position_y")]
    #[get(copied, prop = "position.z", ty = f32, name = "position_z")]
    #[get(prop = "rotation", ty = &Vector3, name = "rotation")]
    #[get(copied, prop = "rotation.z", ty = f32, name = "rotation_2d")]
    #[get(copied, prop = "rotation.x", ty = f32, name = "rotation_x")]
    #[get(copied, prop = "rotation.y", ty = f32, name = "rotation_y")]
    #[get(copied, prop = "rotation.z", ty = f32, name = "rotation_z")]
    #[get(prop = "scale", ty = &Vector3, name = "scale")]
    #[get(copied, prop = "scale", name = "scale_2d", pre = truncate, ty = Vector2)]
    #[get(copied, prop = "scale.x", ty = f32, name = "scale_x")]
    #[get(copied, prop = "scale.y", ty = f32, name = "scale_y")]
    #[get(copied, prop = "scale.z", ty = f32, name = "scale_z")]
    #[get(mut, also = self.tracker |= Self::transform_f())]
    #[get(mut, prop = "position", ty = &mut Vector3, name = "position_mut", also = self.tracker |= Self::transform_f())]
    #[get(mut, prop = "position.x", ty = &mut f32, name = "position_x_mut", also = self.tracker |= Self::transform_f())]
    #[get(mut, prop = "position.y", ty = &mut f32, name = "position_y_mut", also = self.tracker |= Self::transform_f())]
    #[get(mut, prop = "position.z", ty = &mut f32, name = "position_z_mut", also = self.tracker |= Self::transform_f())]
    #[get(mut, prop = "rotation", ty = &mut Vector3, name = "rotation_mut", also = self.tracker |= Self::transform_f())]
    #[get(mut, prop = "rotation.x", ty = &mut f32, name = "rotation_x_mut", also = self.tracker |= Self::transform_f())]
    #[get(mut, prop = "rotation.y", ty = &mut f32, name = "rotation_y_mut", also = self.tracker |= Self::transform_f())]
    #[get(mut, prop = "rotation.z", ty = &mut f32, name = "rotation_z_mut", also = self.tracker |= Self::transform_f())]
    #[get(mut, prop = "scale", ty = &mut Vector3, name = "scale_mut", also = self.tracker |= Self::transform_f())]
    #[get(mut, prop = "scale.x", ty = &mut f32, name = "scale_x_mut", also = self.tracker |= Self::transform_f())]
    #[get(mut, prop = "scale.y", ty = &mut f32, name = "scale_y_mut", also = self.tracker |= Self::transform_f())]
    #[get(mut, prop = "scale.z", ty = &mut f32, name = "scale_z_mut", also = self.tracker |= Self::transform_f())]
    #[set(also = self.tracker |= Self::transform_f())]
    #[set(into, prop = "position", ty = Vector3, name = "set_position", also = self.tracker |= Self::transform_f())]
    #[set(prop = "position.x", ty = f32, name = "set_position_x", also = self.tracker |= Self::transform_f())]
    #[set(prop = "position.y", ty = f32, name = "set_position_y", also = self.tracker |= Self::transform_f())]
    #[set(prop = "position.z", ty = f32, name = "set_position_z", also = self.tracker |= Self::transform_f())]
    #[set(into, prop = "rotation", ty = Vector3, name = "set_rotation", also = self.tracker |= Self::transform_f())]
    #[set(prop = "rotation.z", ty = f32, name = "set_rotation_2d", also = self.tracker |= Self::transform_f())]
    #[set(prop = "rotation.x", ty = f32, name = "set_rotation_x", also = self.tracker |= Self::transform_f())]
    #[set(prop = "rotation.y", ty = f32, name = "set_rotation_y", also = self.tracker |= Self::transform_f())]
    #[set(prop = "rotation.z", ty = f32, name = "set_rotation_z", also = self.tracker |= Self::transform_f())]
    #[set(into, prop = "scale", ty = Vector3, name = "set_scale", also = self.tracker |= Self::transform_f())]
    #[set(prop = "scale.x", ty = f32, name = "set_scale_x", also = self.tracker |= Self::transform_f())]
    #[set(prop = "scale.y", ty = f32, name = "set_scale_y", also = self.tracker |= Self::transform_f())]
    #[set(prop = "scale.z", ty = f32, name = "set_scale_z", also = self.tracker |= Self::transform_f())]
    transform: Transform, // Group's world transfor
}

impl MeshGroup {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.tracker |= Self::transform_f();
        self.transform = transform;
        self
    }

    pub fn add(&mut self, handle: MeshHandle, local_transform: Transform) {
        self.tracker |= Self::transform_f();
        self.entries.push(MeshEntry {
            handle,
            local_transform,
        });
    }

    pub fn remove(&mut self, handle: &MeshHandle) {
        self.entries.retain(|e| &e.handle != handle);
    }

    pub fn set_local_transform(&mut self, handle: &MeshHandle, transform: Transform) {
        if let Some(entry) = self.entries.iter_mut().find(|e| &e.handle == handle) {
            entry.local_transform = transform;
        }
    }

    pub fn update(&mut self, renderer: &mut Renderer) {
        if self.is_dirty(Self::transform_f()) {
            for entry in &self.entries {
                let mesh = renderer.get_mesh_mut(entry.handle);
                mesh.set_transform(self.transform.combine(&entry.local_transform));
            }

            self.clear_dirty(Self::transform_f());
        }
    }
}
