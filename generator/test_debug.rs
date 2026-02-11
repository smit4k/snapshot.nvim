use chrono::offset::Local;
use chrono::DateTime;

fn default_output_path() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let now: DateTime<Local> = Local::now();
    let formatted_time = now.format("%Y-%m-%d_%H-%M-%S").to_string();
    let result = format!("{}/snapshot-{}.png", home, formatted_time);
    eprintln!("DEBUG: HOME = {}", home);
    eprintln!("DEBUG: default_output_path() returns: {}", result);
    result
}

fn main() {
    let path = default_output_path();
    println!("Path before expansion: {}", path);
    
    let expanded = shellexpand::full(&path).unwrap();
    println!("Path after expansion: {}", expanded);
}
