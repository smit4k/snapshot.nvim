fn main() {
    println!("HOME env var: {:?}", std::env::var("HOME"));
    println!("dirs::home_dir(): {:?}", dirs::home_dir());
    println!("dirs::picture_dir(): {:?}", dirs::picture_dir());
}
