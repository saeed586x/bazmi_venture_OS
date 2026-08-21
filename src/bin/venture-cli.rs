//! Minimal Venture OS CLI - Calls real Kernel and emits valid ExecutionPlan.v1 JSON

use venture_os_kernel::Kernel;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <idea>", args[0]);
        std::process::exit(1);
    }

    let idea = &args[1];
    let kernel = Kernel::new();

    match kernel.process_intent(idea) {
        Ok(plan) => {
            let json = serde_json::to_string_pretty(&plan).expect("Failed to serialize plan");
            println!("{}", json);
        }
        Err(e) => {
            eprintln!("Error processing intent: {}", e);
            std::process::exit(1);
        }
    }
}
