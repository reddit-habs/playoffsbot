mod nhlapi;

fn main() {
    println!("Hello, world!");
    let s = nhlapi::schedule::today().unwrap();
    println!("{:#?}", s);
}
