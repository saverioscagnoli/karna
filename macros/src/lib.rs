/// For things to implement the `Mesh` trait (see renderer/src/mesh.rs)
/// They must implement Deref<Target = InstanceData>
#[macro_export]
macro_rules! impl_mesh_deref {
    ($type:ty) => {
        impl Deref for $type {
            type Target = InstanceData;
            fn deref(&self) -> &Self::Target {
                &self.instance_data
            }
        }

        impl DerefMut for $type {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.instance_data
            }
        }
    };
}
