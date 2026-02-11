fn main() {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    println!("HOME from env::var: {}", home);
    
    let path = format!("{}/snapshot-test.png", home);
    println!("Formatted path: {}", path);
    
    let expanded = shellexpand::full(&path).unwrap();
    println!("After shellexpand: {}", expanded);
}
