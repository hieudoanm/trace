mod shapes;

use shapes::{total_area, Circle, Rectangle, Shape};

fn main() {
    let shapes: Vec<&dyn Shape> = vec![
        &Circle { radius: 2.0 },
        &Rectangle {
            width: 3.0,
            height: 4.0,
        },
    ];
    println!("Total area: {:.2}", total_area(&shapes));
}
