use canary_pinout::PinoutService;
use canary_models::embedded::ModuleType;

#[test]
fn test_list_manufacturers() {
    let manufacturers = PinoutService::list_manufacturers();
    assert!(manufacturers.len() >= 6);
    assert!(manufacturers.contains(&"vw"));
}

#[test]
fn test_get_ecus_by_manufacturer() {
    // This will be empty in Phase 1, populated in Phase 2
    let result = PinoutService::get_ecus_by_manufacturer("vw");
    assert!(result.is_ok());
}

#[test]
fn test_get_ecus_by_module_type() {
    let result = PinoutService::get_ecus_by_module_type(ModuleType::ECM);
    assert!(result.is_ok());
}

#[test]
fn test_get_ecu_by_id() {
    // Phase 1: Should return Err with empty data
    let result = PinoutService::get_ecu_by_id("vw_golf_mk7_ecm");
    assert!(result.is_err()); // Empty data in Phase 1
}

#[test]
fn test_get_ecu_by_id_invalid_format() {
    // Edge case: malformed ID (empty string)
    let result = PinoutService::get_ecu_by_id("");
    assert!(result.is_err());
    // Empty string splits to "", which is treated as an unknown manufacturer
    assert!(result.unwrap_err().to_string().contains("Manufacturer"));
}

#[test]
fn test_get_ecu_by_id_unknown_manufacturer() {
    let result = PinoutService::get_ecu_by_id("invalid_model_2020_ecm");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Manufacturer"));
}

#[test]
fn test_get_ecus_by_invalid_manufacturer() {
    let result = PinoutService::get_ecus_by_manufacturer("invalid_mfr");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Manufacturer"));
}
