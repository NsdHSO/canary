use canary_core::{DtcService, embedded::DtcSystem};

fn main() -> Result<(), canary_core::CanaryError> {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Canary DTC Analysis Examples");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Parse DTC systems from codes
    println!("🔍 Parsing DTC Systems");
    println!("───────────────────────────────────────────────────────");

    let test_codes = vec!["P0301", "B0001", "C0123", "U0001", "P0420"];

    for code in test_codes {
        match DtcService::parse_system(code) {
            Ok(system) => {
                let system_name = match system {
                    DtcSystem::Powertrain => "Powertrain (Engine/Transmission)",
                    DtcSystem::Body => "Body (Doors, Windows, Seats)",
                    DtcSystem::Chassis => "Chassis (Brakes, Steering, Suspension)",
                    DtcSystem::Network => "Network (CAN bus, Communication)",
                };
                println!("  {} → {:?}: {}", code, system, system_name);
            }
            Err(e) => {
                println!("  {} → ❌ Error: {}", code, e);
            }
        }
    }

    // Lookup specific DTCs
    println!("\n\n⚠️  Detailed DTC Lookup");
    println!("───────────────────────────────────────────────────────");

    let codes_to_lookup = vec!["P0301", "P0302", "P0420", "P0171"];

    for code in codes_to_lookup {
        match DtcService::lookup_code(code) {
            Ok(dtc) => {
                println!("\n📋 Code: {}", dtc.code);
                println!("   System: {:?}", dtc.system);
                println!("   Description: {}", dtc.description);
            }
            Err(_) => {
                println!("\n❌ Code {} not found in database", code);
            }
        }
    }

    // Search by description keyword
    println!("\n\n🔎 Keyword Search: 'misfire'");
    println!("───────────────────────────────────────────────────────");

    let misfire_codes = DtcService::search_by_description("misfire");
    println!("Found {} codes related to 'misfire':\n", misfire_codes.len());

    for dtc in misfire_codes {
        println!("  • {} - {}", dtc.code, dtc.description);
    }

    // Get all codes by system
    println!("\n\n📊 All Powertrain Codes");
    println!("───────────────────────────────────────────────────────");

    let powertrain_codes = DtcService::get_by_system(DtcSystem::Powertrain);
    println!("Total powertrain codes in database: {}\n", powertrain_codes.len());

    for dtc in powertrain_codes {
        println!("  • {} - {}", dtc.code, dtc.description);
    }

    // List all available codes
    println!("\n\n📝 Complete DTC Database");
    println!("───────────────────────────────────────────────────────");

    let all_codes = DtcService::list_all();
    println!("Total codes in database: {}", all_codes.len());

    // Group by system
    let mut systems: std::collections::HashMap<DtcSystem, Vec<_>> = std::collections::HashMap::new();
    for dtc in all_codes {
        systems.entry(dtc.system).or_default().push(dtc);
    }

    println!("\nBreakdown by system:");
    for (system, codes) in systems {
        println!("  {:?}: {} codes", system, codes.len());
    }

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ DTC analysis demonstration complete!");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    Ok(())
}
