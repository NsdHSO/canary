use canary_core::ServiceProcedureService;

fn main() -> Result<(), canary_core::CanaryError> {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Canary Service Procedures Examples");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // List all available procedures
    println!("📋 Available Service Procedures");
    println!("───────────────────────────────────────────────────────");

    let all_procedures = ServiceProcedureService::list_all();
    println!("Total procedures in database: {}\n", all_procedures.len());

    for proc in &all_procedures {
        let time = proc
            .estimated_time_minutes
            .map(|t| format!("{} min", t))
            .unwrap_or_else(|| "varies".to_string());

        println!("  • {} ({:?}) - {}", proc.name, proc.category, time);
    }

    // Get procedures by category
    println!("\n\n🔧 Maintenance Procedures");
    println!("───────────────────────────────────────────────────────");

    let maintenance = ServiceProcedureService::get_maintenance_procedures();
    println!("Found {} maintenance procedures:\n", maintenance.len());

    for proc in maintenance {
        println!("  ✓ {}", proc.name);
        println!("    Tools: {}", proc.tools_required.join(", "));
    }

    // Search by time range
    println!("\n\n⏱️  Quick Procedures (< 35 minutes)");
    println!("───────────────────────────────────────────────────────");

    let quick_procedures = ServiceProcedureService::get_by_time_range(0, 35);
    for proc in quick_procedures {
        println!("  • {} - {} minutes", proc.name, proc.estimated_time_minutes.unwrap_or(0));
    }

    // Detailed procedure walkthrough - Oil Change
    println!("\n\n🛢️  Detailed Procedure: Oil Change");
    println!("───────────────────────────────────────────────────────");

    let oil_change = ServiceProcedureService::get_procedure("oil_change")?;
    println!("Name: {}", oil_change.name);
    println!("Category: {:?}", oil_change.category);
    println!("Description: {}", oil_change.description);
    println!("Estimated time: {} minutes", oil_change.estimated_time_minutes.unwrap_or(0));
    println!("Tools required:");
    for tool in &oil_change.tools_required {
        println!("  • {}", tool);
    }

    println!("\nStep-by-step instructions:");
    for step in &oil_change.steps {
        println!("\n  Step {}: {}", step.order, step.instruction);
        if !step.warnings.is_empty() {
            for warning in &step.warnings {
                println!("    ⚠️  WARNING: {}", warning);
            }
        }
    }

    // Detailed procedure walkthrough - Brake Bleeding
    println!("\n\n🔴 Detailed Procedure: Brake Bleeding");
    println!("───────────────────────────────────────────────────────");

    let brake_bleeding = ServiceProcedureService::get_procedure("brake_bleeding")?;
    println!("Name: {}", brake_bleeding.name);
    println!("Category: {:?}", brake_bleeding.category);
    println!("Description: {}", brake_bleeding.description);
    println!("Estimated time: {} minutes", brake_bleeding.estimated_time_minutes.unwrap_or(0));
    println!("Total steps: {}", brake_bleeding.steps.len());

    println!("\nKey safety warnings:");
    let mut warning_count = 0;
    for step in &brake_bleeding.steps {
        for warning in &step.warnings {
            warning_count += 1;
            println!("  ⚠️  {}", warning);
        }
    }
    println!("\nTotal safety warnings: {}", warning_count);

    // Search by name
    println!("\n\n🔎 Search: 'brake'");
    println!("───────────────────────────────────────────────────────");

    let brake_procedures = ServiceProcedureService::search_by_name("brake");
    println!("Found {} procedures matching 'brake':\n", brake_procedures.len());

    for proc in brake_procedures {
        println!("  • {} - {}", proc.id, proc.name);
    }

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ Service procedures demonstration complete!");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    Ok(())
}
