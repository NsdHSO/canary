use canary_core::PinoutService;
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct EcuArgs {
    #[command(subcommand)]
    pub command: EcuCommand,
}

#[derive(Subcommand)]
pub enum EcuCommand {
    /// Show detailed ECU information
    Show {
        /// ECU ID (e.g., vw_golf_mk7_2015_ecm_med1725)
        id: String,
    },
    /// List ECUs (optionally filtered by manufacturer)
    List {
        /// Filter by manufacturer (e.g., vw, gm, ford)
        #[arg(short, long)]
        manufacturer: Option<String>,
    },
    /// Search ECUs by name or description
    Search {
        /// Search query
        query: String,
    },
}

pub fn handle_ecu(args: EcuArgs) -> Result<(), Box<dyn std::error::Error>> {
    match args.command {
        EcuCommand::Show { id } => show_ecu(&id),
        EcuCommand::List { manufacturer } => list_ecus(manufacturer.as_deref()),
        EcuCommand::Search { query } => search_ecus(&query),
    }
}

fn show_ecu(id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let ecu = PinoutService::get_ecu_by_id(id)?;

    println!("ECU Details: {}", ecu.id);
    println!("══════════════════════════════════════════════════════");
    println!("Module Type: {:?}", ecu.module_type);
    println!(
        "Manufacturer: {} (ECU by {})",
        ecu.manufacturer_id, ecu.ecu_manufacturer
    );

    if !ecu.part_numbers.is_empty() {
        println!("Part Numbers: {}", ecu.part_numbers.join(", "));
    }

    // Vehicle compatibility
    if !ecu.vehicle_models.is_empty() {
        println!("\nVehicle Compatibility:");
        for vehicle in &ecu.vehicle_models {
            let years_str = if vehicle.years.len() > 1 {
                format!("{}-{}", vehicle.years.first().unwrap(), vehicle.years.last().unwrap())
            } else {
                vehicle.years.first().map(|y| y.to_string()).unwrap_or_default()
            };

            if let Some(engine) = &vehicle.engine {
                println!("  {} {} ({}, {})", vehicle.manufacturer, vehicle.model, years_str, engine);
            } else {
                println!("  {} {} ({})", vehicle.manufacturer, vehicle.model, years_str);
            }
        }
    }

    // Connectors
    println!("\nConnectors: {}", ecu.connectors.len());
    for connector in &ecu.connectors {
        let pin_count = connector.pins.len();
        println!("  {} - {} ({} pins)", connector.connector_id, connector.connector_type, pin_count);
    }

    // Power requirements
    println!("\nPower Requirements:");
    let power = &ecu.power_requirements;
    println!(
        "  Voltage: {}-{}V (nominal: {}V)",
        power.voltage_min, power.voltage_max, power.voltage_nominal
    );
    if let Some(current) = power.current_max {
        println!("  Max Current: {}A", current);
    }
    if let Some(fuse) = power.fuse_rating {
        println!("  Fuse Rating: {}A", fuse);
    }

    // Protocols
    if !ecu.supported_protocols.is_empty() {
        println!("\nProtocols: {}", ecu.supported_protocols.join(", "));
    }

    // Memory specs
    if let Some(memory) = &ecu.flash_memory {
        println!("\nMemory:");
        println!("  Flash: {} KB", memory.flash_size_kb);
        println!("  RAM: {} KB", memory.ram_size_kb);
        if let Some(eeprom) = memory.eeprom_size_kb {
            println!("  EEPROM: {} KB", eeprom);
        }
        println!("  CPU: {}", memory.cpu);
    }

    Ok(())
}

fn list_ecus(manufacturer: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(mfr) = manufacturer {
        // List ECUs for specific manufacturer
        let ecus = PinoutService::get_ecus_by_manufacturer(mfr)?;

        println!("ECUs for manufacturer '{}':", mfr);
        println!("══════════════════════════════════════════════════════");

        if ecus.is_empty() {
            println!("\nNo ECUs found for manufacturer '{}'", mfr);
        } else {
            println!("\nFound {} ECU(s):\n", ecus.len());
            for ecu in ecus {
                println!("  {} ({:?})", ecu.id, ecu.module_type);
                println!("    Manufacturer: {}", ecu.ecu_manufacturer);
                if !ecu.part_numbers.is_empty() {
                    println!("    Part Numbers: {}", ecu.part_numbers.join(", "));
                }
                println!();
            }
        }
    } else {
        // List all ECUs from all manufacturers
        let manufacturers = PinoutService::list_manufacturers();

        println!("All available ECUs:");
        println!("══════════════════════════════════════════════════════");

        let mut total_count = 0;
        for mfr in manufacturers {
            if let Ok(ecus) = PinoutService::get_ecus_by_manufacturer(mfr) {
                if !ecus.is_empty() {
                    println!("\n{}:", mfr.to_uppercase());
                    for ecu in ecus {
                        println!("  {} ({:?}, {})", ecu.id, ecu.module_type, ecu.ecu_manufacturer);
                        total_count += 1;
                    }
                }
            }
        }

        println!("\nTotal: {} ECUs", total_count);
    }

    Ok(())
}

fn search_ecus(query: &str) -> Result<(), Box<dyn std::error::Error>> {
    let query_lower = query.to_lowercase();
    let manufacturers = PinoutService::list_manufacturers();
    let mut results = Vec::new();

    // Search across all manufacturers
    for mfr in manufacturers {
        if let Ok(ecus) = PinoutService::get_ecus_by_manufacturer(mfr) {
            for ecu in ecus {
                // Search in ID, name, and manufacturer
                if ecu.id.to_lowercase().contains(&query_lower)
                    || ecu.name.to_lowercase().contains(&query_lower)
                    || ecu.ecu_manufacturer.to_lowercase().contains(&query_lower)
                {
                    results.push(ecu);
                }
            }
        }
    }

    println!("ECU Search Results for '{}':", query);
    println!("══════════════════════════════════════════════════════");

    if results.is_empty() {
        println!("\nNo ECUs found matching '{}'", query);
    } else {
        println!("\nFound {} ECU(s):\n", results.len());
        for ecu in results {
            println!("  {} ({:?})", ecu.id, ecu.module_type);
            println!("    Name: {}", ecu.name);
            println!("    Manufacturer: {}", ecu.ecu_manufacturer);
            println!();
        }
    }

    Ok(())
}
