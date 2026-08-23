//! Minimal Venture OS CLI

use venture_os_kernel::core::kernel::Kernel;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <idea>", args[0]);
        std::process::exit(1);
    }

    let idea = &args[1];
    println!("Processing idea: {}", idea);

    // Initialize kernel and process the intent
    let kernel = Kernel::new();

    // Process through kernel to generate execution plan
    match kernel.process_intent(idea) {
        Ok(plan) => {
            println!("Execution plan generated successfully");
            // Output valid ExecutionPlan.v1 JSON
            match serde_json::to_string_pretty(&plan) {
                Ok(json) => println!("{}", json),
                Err(e) => eprintln!("Error serializing plan: {}", e),
            }
        }
        Err(e) => {
            eprintln!("Error processing intent: {}", e);
            std::process::exit(1);
        }
    }
}
