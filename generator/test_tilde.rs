fn main() {
    eprintln!("Test 1: Using $HOME");
    let home = std::env::var("HOME").unwrap();
    let path1 = format!("{}/snapshot.png", home);
    println!("  Before: {}", path1);
    println!("  After:  {}", shellexpand::full(&path1).unwrap());
    
    eprintln!("\nTest 2: Using ~");
    let path2 = "~/snapshot.png";
    println!("  Before: {}", path2);
    println!("  After:  {}", shellexpand::full(path2).unwrap());
    
    eprintln!("\nTest 3: Using tilde without slash");
    let path3 = "~";
    println!("  Before: {}", path3);
    println!("  After:  {}", shellexpand::full(path3).unwrap());
}
