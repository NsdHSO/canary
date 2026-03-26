use canary_core::{DtcService, PinoutService, ProtocolDecoder, ProtocolFactory, ServiceProcedureService};

fn main() -> Result<(), canary_core::CanaryError> {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Canary Automotive Reverse Engineering Library");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // 1. Pin Mapping Lookup
    println!("📌 OBD-II J1962 Pin Mapping:");
    println!("───────────────────────────────────────────────────────");
    let obd2 = PinoutService::get_obd2_pinout()?;
    println!("Connector: {}", obd2.connector_type);
    println!("Total pins: {}", obd2.pins.len());
    println!("\nKey pins:");
    println!("  Pin 6  - {}", obd2.pins[5].signal_name);
    println!("  Pin 14 - {}", obd2.pins[13].signal_name);
    println!("  Pin 16 - {}", obd2.pins[15].signal_name);

    // 2. Protocol Decoding
    println!("\n🔌 CAN Bus Protocol Decoding:");
    println!("───────────────────────────────────────────────────────");
    let decoder = ProtocolFactory::create_can_decoder()?;
    let raw_bytes = vec![0x00, 0x00, 0x01, 0x23, 0x01, 0x02, 0x03];
    let frame = decoder.decode(&raw_bytes)?;
    println!("CAN ID: 0x{:X}", frame.id);
    println!("Data: {:?}", frame.data);
    println!("Timestamp: {}", frame.timestamp);

    // 3. DTC Lookup
    println!("\n⚠️  Diagnostic Trouble Codes:");
    println!("───────────────────────────────────────────────────────");
    let dtc = DtcService::lookup_code("P0301")?;
    println!("Code: {}", dtc.code);
    println!("System: {:?}", dtc.system);
    println!("Description: {}", dtc.description);

    println!("\nSearching for 'misfire' DTCs:");
    let misfire_codes = DtcService::search_by_description("misfire");
    for code in misfire_codes {
        println!("  {} - {}", code.code, code.description);
    }

    // 4. Service Procedures
    println!("\n🔧 Service Procedures:");
    println!("───────────────────────────────────────────────────────");
    let procedure = ServiceProcedureService::get_procedure("oil_change")?;
    println!("Procedure: {}", procedure.name);
    println!("Category: {:?}", procedure.category);
    println!("Estimated time: {} minutes", procedure.estimated_time_minutes.unwrap_or(0));
    println!("Steps: {} total", procedure.steps.len());
    println!("\nFirst 3 steps:");
    for step in procedure.steps.iter().take(3) {
        println!("  {}. {}", step.order, step.instruction);
    }

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ All features working correctly!");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    Ok(())
}
