use canary_core::{embedded::ModuleType, PinoutService};
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct ModuleArgs {
    #[command(subcommand)]
    pub command: ModuleCommand,
}

#[derive(Subcommand)]
pub enum ModuleCommand {
    /// List ECUs by module type
    List {
        /// Module type (ECM, PCM, TCM, BCM, etc.)
        module_type: String,
    },
}

pub fn handle_module(args: ModuleArgs) -> Result<(), Box<dyn std::error::Error>> {
    match args.command {
        ModuleCommand::List { module_type } => list_by_module_type(&module_type),
    }
}

fn list_by_module_type(module_type_str: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Parse module type (case-insensitive)
    let module_type = parse_module_type(module_type_str)?;

    let ecus = PinoutService::get_ecus_by_module_type(module_type)?;

    println!("ECUs with module type '{:?}':", module_type);
    println!("══════════════════════════════════════════════════════");

    if ecus.is_empty() {
        println!("\nNo ECUs found with module type '{:?}'", module_type);
    } else {
        for ecu in ecus {
            println!("  {} ({})", ecu.id, ecu.ecu_manufacturer);
        }
    }

    Ok(())
}

fn parse_module_type(s: &str) -> Result<ModuleType, Box<dyn std::error::Error>> {
    match s.to_uppercase().as_str() {
        "ECM" => Ok(ModuleType::ECM),
        "PCM" => Ok(ModuleType::PCM),
        "TCM" => Ok(ModuleType::TCM),
        "BCM" => Ok(ModuleType::BCM),
        "DDM" => Ok(ModuleType::DDM),
        "PDM" => Ok(ModuleType::PDM),
        "HVAC" => Ok(ModuleType::HVAC),
        "ABS" => Ok(ModuleType::ABS),
        "SRS" => Ok(ModuleType::SRS),
        "EPB" => Ok(ModuleType::EPB),
        "IPC" => Ok(ModuleType::IPC),
        "INFOCENTER" => Ok(ModuleType::InfoCenter),
        "GATEWAY" => Ok(ModuleType::Gateway),
        "TELEMATICS" => Ok(ModuleType::Telematics),
        "OBD" => Ok(ModuleType::OBD),
        _ => Err(format!(
            "Invalid module type '{}'. Valid types: ECM, PCM, TCM, BCM, DDM, PDM, HVAC, ABS, SRS, EPB, IPC, InfoCenter, Gateway, Telematics, OBD",
            s
        )
        .into()),
    }
}
