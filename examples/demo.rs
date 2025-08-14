use karna::App;

fn main() {
    App::new()
        .with_size((1024, 768))
        .run()
        .expect("Failed to run application");
}
