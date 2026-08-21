//! Minimal Venture OS CLI

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <idea>", args[0]);
        std::process::exit(1);
    }
    
    let idea = &args[1];
    println!("Processing idea: {}", idea);
    println!("Mock execution plan generated");
    println!("Plan ID: mock-123");
}
