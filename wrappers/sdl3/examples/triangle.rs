use sdl3::SdlGuard;
use sdl3::events::SdlEvent;
use sdl3::events::SdlWindowEvent;
use sdl3::events::poll;
use sdl3::gpu::BufferUsage;
use sdl3::gpu::Device;
use sdl3::gpu::GpuBuffer;
use sdl3::gpu::GraphicsPipeline;
use sdl3::gpu::LoadOp;
use sdl3::gpu::PipelineDesc;
use sdl3::gpu::Sampler;
use sdl3::gpu::SamplerDesc;
use sdl3::gpu::Texture;
use sdl3::gpu::TextureDesc;
use sdl3::gpu::VertexAttribute;
use sdl3::gpu::VertexBuffer;
use sdl3::render::Color;
use sdl3::shadercross::CompileOptions;
use sdl3::shadercross::ShaderCross;
use sdl3::shadercross::ShaderSource;

const VERT: &[u8] = include_bytes!("../../../shaders/immediate.vert.spv");
const FRAG: &[u8] = include_bytes!("../../../shaders/immediate.frag.spv");

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 4],
    uv: [f32; 2],
    page: f32,
}

const IDENTITY: [[f32; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

fn main() {
    let frames: Option<u32> = std::env::args()
        .skip_while(|a| a != "--frames")
        .nth(1)
        .and_then(|n| n.parse().ok());

    let _sdl = SdlGuard::init().expect("SDL init");
    let device = Device::init().expect("GPU device");

    println!("driver: {}", device.driver());
    println!("shader formats: {:?}", device.shader_formats());

    let window = device
        .create_window("triangle", (640u32, 480u32), true)
        .expect("window");

    let shadercross = ShaderCross::init().expect("shadercross");

    let vertex = shadercross
        .create_shader(
            device.share(),
            ShaderSource::Spirv(VERT),
            &CompileOptions::vertex(),
        )
        .expect("vertex shader");

    let fragment = shadercross
        .create_shader(
            device.share(),
            ShaderSource::Spirv(FRAG),
            &CompileOptions::fragment(),
        )
        .expect("fragment shader");

    let pipeline = GraphicsPipeline::new(
        device.share(),
        PipelineDesc::new(&vertex, &fragment)
            .with_vertex_layout(
                &[VertexBuffer::of::<Vertex>(0)],
                &[
                    VertexAttribute::float3(0, 0),
                    VertexAttribute::float4(1, 12),
                    VertexAttribute::float2(2, 28),
                    VertexAttribute::float(3, 36),
                ],
            )
            .with_targets(&[device.swapchain_format(&window)]),
    )
    .expect("pipeline");

    let mut vertices = GpuBuffer::<Vertex>::new(device.share(), "triangle", 3, BufferUsage::VERTEX);

    device
        .upload(
            &mut vertices,
            &[
                Vertex {
                    position: [0.0, 0.6, 0.0],
                    color: [1.0, 0.2, 0.2, 1.0],
                    uv: [0.5, 0.0],
                    page: 0.0,
                },
                Vertex {
                    position: [-0.6, -0.5, 0.0],
                    color: [0.2, 1.0, 0.3, 1.0],
                    uv: [0.0, 1.0],
                    page: 0.0,
                },
                Vertex {
                    position: [0.6, -0.5, 0.0],
                    color: [0.3, 0.4, 1.0, 1.0],
                    uv: [1.0, 1.0],
                    page: 0.0,
                },
            ],
        )
        .expect("upload");

    let white = Texture::new(device.share(), "white", TextureDesc::rgba8_array(1, 1, 1));

    device
        .upload_texture(&white, &[255u8, 255, 255, 255])
        .expect("upload white texel");
    let sampler = Sampler::new(device.share(), SamplerDesc::nearest());

    let mut drawn = 0u32;

    'running: loop {
        for event in poll() {
            match event {
                SdlEvent::Quit => break 'running,
                SdlEvent::Window {
                    wevent: SdlWindowEvent::CloseRequested,
                    ..
                } => break 'running,
                _ => {}
            }
        }

        let Some(mut frame) = device.begin_frame(&window).expect("frame") else {
            continue;
        };

        {
            let mut pass = frame
                .render_pass(LoadOp::Clear(Color::hex(0x101418)))
                .expect("render pass");

            pass.bind_pipeline(&pipeline);
            pass.bind_vertex_buffer(0, &vertices, 0);
            pass.bind_fragment_sampler(0, &white, &sampler);
            pass.push_vertex_uniform(0, &IDENTITY);
            pass.draw(vertices.len() as u32);
        }

        frame.submit().expect("submit");

        drawn += 1;

        if frames.is_some_and(|max| drawn >= max) {
            break;
        }
    }

    println!("drew {drawn} frames");
}
