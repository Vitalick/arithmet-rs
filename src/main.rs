use arithmet::domain::banner;

fn main() {
    for s in banner::render("Hello, world!") {
        println!("{}", s)
    }
}
