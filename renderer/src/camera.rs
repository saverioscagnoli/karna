use gpu::core::GpuBuffer;
use macros::Get;
use macros::Set;
use math::Matrix4;
use math::Size;
use math::Vector2;
use math::Vector3;

pub trait Projection {
    /// Returns the projection matrix that transforms camera/eye space into clip space.
    fn matrix(&self, view: Size<u32>) -> Matrix4;

    /// Returns the view matrix that transforms world space into camera/eye space.
    ///
    /// The default implementation uses a full 3D `look_at` matrix.
    /// Override this for projections that use a different camera model
    /// (e.g. 2D orthographic cameras that only translate).
    fn view_matrix(&self, position: Vector3, target: Vector3, up: Vector3) -> Matrix4 {
        Matrix4::look_at(position, target, up)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct OrthographicProjection {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
    pub near: f32,
    pub far: f32,
}

impl OrthographicProjection {
    pub fn new(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        Self {
            left,
            right,
            bottom,
            top,
            near,
            far,
        }
    }
}

impl Projection for OrthographicProjection {
    fn matrix(&self, _view: Size<u32>) -> Matrix4 {
        Matrix4::orthographic(
            self.left,
            self.right,
            self.bottom,
            self.top,
            self.near,
            self.far,
        )
    }

    /// 2D orthographic cameras only need a simple translation; there is no
    /// concept of "looking at" a target point.
    fn view_matrix(&self, position: Vector3, _target: Vector3, _up: Vector3) -> Matrix4 {
        Matrix4::from_translation(Vector3::new(-position.x, -position.y, 0.0))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PerspectiveProjection {
    /// Vertical field of view in radians.
    pub fov_y: f32,
    pub near: f32,
    pub far: f32,
}

impl PerspectiveProjection {
    pub fn new(fov_y: f32, near: f32, far: f32) -> Self {
        Self { fov_y, near, far }
    }
}

impl Projection for PerspectiveProjection {
    fn matrix(&self, view: Size<u32>) -> Matrix4 {
        let aspect = view.width as f32 / view.height.max(1) as f32;
        Matrix4::perspective(self.fov_y, aspect, self.near, self.far)
    }
}

#[derive(Get, Set)]
pub struct Camera {
    projection: Box<dyn Projection>,
    vp_buffer: GpuBuffer<Matrix4>,

    #[get(visibility = "pub(crate)")]
    vp_bgl: wgpu::BindGroupLayout,

    #[get(visibility = "pub(crate)")]
    vp_bg: wgpu::BindGroup,

    #[get(copied)]
    #[get(mut)]
    #[set(into)]
    position: Vector3,

    #[get(copied)]
    #[get(mut)]
    #[set(into)]
    target: Vector3,
    up: Vector3,
}

impl Camera {
    pub fn new<P: Projection + 'static>(projection: P) -> Self {
        let device = gpu::device();
        let vp_buffer = GpuBuffer::builder()
            .label("View-Projection Buffer")
            .uniform()
            .copy_dst()
            .capacity(1)
            .build();

        let vp_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("View-Projection Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let vp_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("View-Projection Bind Group"),
            layout: &vp_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &vp_buffer.inner(),
                    offset: 0,
                    size: None,
                }),
            }],
        });

        Self {
            projection: Box::new(projection),
            vp_buffer,
            vp_bgl,
            vp_bg,
            position: Vector3::new(0.0, 0.0, -5.0),
            target: Vector3::z(),
            up: Vector3::y(),
        }
    }

    #[inline]
    pub fn projection_mut(&mut self) -> &mut dyn Projection {
        &mut *self.projection
    }

    #[inline]
    pub fn set_projection<P: Projection + 'static>(&mut self, projection: P) {
        self.projection = Box::new(projection);
    }

    #[inline]
    pub fn view_projection(&self, view: Size<u32>) -> Matrix4 {
        let projection = self.projection.matrix(view);
        let view_mat = self
            .projection
            .view_matrix(self.position, self.target, self.up);

        projection * view_mat
    }

    #[inline]
    pub fn update(&self, view: Size<u32>) {
        let vp = self.view_projection(view);
        self.vp_buffer.write(0, &[vp]);
    }

    #[inline]
    pub fn set_position_2d<P: Into<Vector2>>(&mut self, pos: P) {
        let pos_2d: Vector2 = pos.into();
        let pos = pos_2d.extend(self.position.z);

        self.position = pos;
    }
}
