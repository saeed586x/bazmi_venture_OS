use venture_os_kernel::Kernel;

fn main() {
    let kernel = Kernel::new();
    let intent = "Build a customer management system";
    
    match kernel.process_intent(intent) {
        Ok(plan) => {
            println!("=== ExecutionPlan.v1 Validation ===");
            println!("ID (UUID format): {}", plan.id);
            println!("Version (semver): {}", plan.version);
            println!("Goals count (expected 3): {}", plan.goals.len());
            println!("Constraints count (expected 2): {}", plan.constraints.len());
            println!("Required capabilities count (expected 4): {}", plan.required_capabilities.len());
            println!("Tasks count (expected 4): {}", plan.tasks.len());
            println!("Dependencies count (at least 1): {}", plan.dependencies.len());
            println!("Gates count (expected 2): {}", plan.gates.len());
            println!("Completion conditions count (expected 3): {}", plan.completion_conditions.len());
            println!("Provenance present: {}", plan.provenance.is_some());
            
            // Validate all requirements
            let mut passed = true;
            
            if plan.goals.len() != 3 {
                println!("FAIL: Goals should be exactly 3");
                passed = false;
            }
            if plan.constraints.len() != 2 {
                println!("FAIL: Constraints should be exactly 2");
                passed = false;
            }
            if plan.required_capabilities.len() != 4 {
                println!("FAIL: Required capabilities should be exactly 4");
                passed = false;
            }
            if plan.tasks.len() != 4 {
                println!("FAIL: Tasks should be exactly 4");
                passed = false;
            }
            if plan.dependencies.len() < 1 {
                println!("FAIL: Dependencies should be at least 1");
                passed = false;
            }
            if plan.gates.len() != 2 {
                println!("FAIL: Gates should be exactly 2");
                passed = false;
            }
            if plan.completion_conditions.len() != 3 {
                println!("FAIL: Completion conditions should be exactly 3");
                passed = false;
            }
            if plan.provenance.is_none() {
                println!("FAIL: Provenance must be non-null");
                passed = false;
            }
            
            // Check UUID format (basic check)
            if plan.id.is_empty() {
                println!("FAIL: ID cannot be empty");
                passed = false;
            }
            
            // Check semver format
            let parts: Vec<&str> = plan.version.split('.').collect();
            if parts.len() != 3 || !parts.iter().all(|p| p.parse::<u32>().is_ok()) {
                println!("FAIL: Version must be valid semver");
                passed = false;
            }
            
            if passed {
                println!("\n=== ALL ISSUE-02 REQUIREMENTS PASSED ===");
            } else {
                println!("\n=== SOME REQUIREMENTS FAILED ===");
                std::process::exit(1);
            }
        }
        Err(e) => {
            println!("ERROR: Failed to process intent: {:?}", e);
            std::process::exit(1);
        }
    }
}
