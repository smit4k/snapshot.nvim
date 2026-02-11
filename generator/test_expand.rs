fn main() {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    println!("HOME env var: {}", home);
    
    let test_path = format!("{}/snapshot-test.png", home);
    println!("Original path: {}", test_path);
    
    let expanded = shellexpand::full(&test_path).unwrap();
    println!("After shellexpand::full: {}", expanded);
}
