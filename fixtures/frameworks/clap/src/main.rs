use clap::Command;

fn main() {
    let matches = Command::new("myapp")
        .about("My CLI application")
        .get_matches();
    println!("{:?}", matches);
}
