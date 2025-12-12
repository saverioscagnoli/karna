use macros::dirty;

#[dirty]
struct Transform {
    #[dirty]
    x: f32,
    #[dirty]
    y: f32,
    #[dirty]
    z: f32,
    name: String,
}

#[dirty]
struct Config {
    #[dirty(into)]
    name: String,
    #[dirty]
    value: i32,
}

fn main() {
    // Example 1: Basic dirty tracking
    let mut transform = Transform {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        name: "player".to_string(),
        __tracker: 0,
    };

    println!("Initial tracker: {:#08b}", transform.__tracker);

    // Using set_x - only marks dirty if value changes
    transform.set_x(1.0);
    println!("After set_x(1.0): {:#08b}", transform.__tracker);
    println!("x bit mask: {:#08b}", Transform::x());

    // Setting same value again - should not mark dirty
    transform.set_x(1.0);
    println!("After set_x(1.0) again: {:#08b}", transform.__tracker);

    // Using y_mut - always marks dirty
    *transform.y_mut() = 2.0;
    println!("After y_mut(): {:#08b}", transform.__tracker);
    println!("y bit mask: {:#08b}", Transform::y());

    // Setting z
    transform.set_z(3.0);
    println!("After set_z(3.0): {:#08b}", transform.__tracker);
    println!("z bit mask: {:#08b}", Transform::z());

    // Check individual bits
    if transform.__tracker & Transform::x() != 0 {
        println!("x was modified!");
    }
    if transform.__tracker & Transform::y() != 0 {
        println!("y was modified!");
    }
    if transform.__tracker & Transform::z() != 0 {
        println!("z was modified!");
    }

    // Example 2: Using Into
    let mut config = Config {
        name: "default".to_string(),
        value: 0,
        __tracker: 0,
    };

    println!("\nConfig tracker: {:#08b}", config.__tracker);

    // Can pass &str thanks to #[dirty(into)]
    config.set_name("new_name");
    println!("After set_name: {:#08b}", config.__tracker);

    config.set_value(42);
    println!("After set_value: {:#08b}", config.__tracker);

    // Clear tracker
    config.__tracker = 0;
    println!("After clearing: {:#08b}", config.__tracker);
}
