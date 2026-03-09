use karna::App;
use karna::WindowBuilder;
use karna::math::Size;

fn main() {
    App::builder()
        .with_window(WindowBuilder::new().with_size(Size::new(1280, 720)))
        .build()
        .run();
}
