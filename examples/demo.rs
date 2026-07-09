use karna::App;
use karna::WindowBuilder;

fn main() {
    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_title("demo")
                .with_size((1280, 720)),
        )
        .build()
        .run();
}
