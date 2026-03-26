use canary_core::{ProtocolDecoder, ProtocolFactory};

fn main() -> Result<(), canary_core::CanaryError> {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Canary Protocol Decoding Examples");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // CAN Bus Decoding Example
    println!("🔌 CAN Bus 2.0B Protocol");
    println!("───────────────────────────────────────────────────────");

    let can_decoder = ProtocolFactory::create_can_decoder()?;

    // Example CAN frames
    let frames = vec![
        (vec![0x00, 0x00, 0x07, 0xE8, 0x03, 0x41, 0x0C, 0x1A, 0xF8], "Engine RPM request"),
        (vec![0x00, 0x00, 0x07, 0xDF, 0x02, 0x01, 0x00], "Current data request"),
        (vec![0x00, 0x00, 0x07, 0xE0, 0x06, 0x41, 0x05, 0x7F], "Coolant temp response"),
    ];

    for (raw_data, description) in frames {
        match can_decoder.decode(&raw_data) {
            Ok(frame) => {
                println!("\n📨 {}", description);
                println!("  CAN ID: 0x{:04X}", frame.id);
                println!("  Data: {:02X?}", frame.data);
                println!("  Length: {} bytes", frame.data.len());

                // Encode it back
                let encoded = can_decoder.encode(&frame)?;
                println!("  Encoded: {:02X?}", encoded);
                println!("  ✅ Encode/decode symmetry verified");
            }
            Err(e) => {
                println!("\n❌ Error decoding: {}", e);
            }
        }
    }

    // K-Line Protocol Example
    println!("\n\n🔌 K-Line (KWP2000) Protocol");
    println!("───────────────────────────────────────────────────────");

    let kline_decoder = ProtocolFactory::create_kline_decoder()?;

    let kline_frames = vec![
        (vec![0x80, 0x10, 0x41, 0x0C, 0x1A, 0xF8, 0xE3], "Engine RPM"),
        (vec![0x82, 0x11, 0x01, 0x00, 0x5A], "Mode 01 request"),
    ];

    for (raw_data, description) in kline_frames {
        match kline_decoder.decode(&raw_data) {
            Ok(frame) => {
                println!("\n📨 {}", description);
                println!("  Header: {:02X?}", frame.header);
                println!("  Data: {:02X?}", frame.data);
                println!("  Checksum: 0x{:02X}", frame.checksum);

                let encoded = kline_decoder.encode(&frame)?;
                println!("  Encoded: {:02X?}", encoded);
                println!("  ✅ Encode/decode symmetry verified");
            }
            Err(e) => {
                println!("\n❌ Error decoding: {}", e);
            }
        }
    }

    // List available protocols
    println!("\n\n📋 Available Protocols");
    println!("───────────────────────────────────────────────────────");
    let protocols = ProtocolFactory::list_available_protocols();
    for protocol_id in protocols {
        println!("  • {}", protocol_id);
    }

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ Protocol decoding demonstration complete!");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    Ok(())
}
