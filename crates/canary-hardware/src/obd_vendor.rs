use std::fmt;
use std::str::FromStr;

/// Known OBD adapter vendors/brands
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObdVendor {
    /// Generic OBD adapters
    OBD,
    /// ELM Electronics (ELM327 chips)
    ELM,
    /// Generic OBD-II adapters
    OBDII,
    /// OBDLink brand adapters
    OBDLink,
    /// Vgate brand adapters
    Vgate,
    /// iCar brand adapters
    ICar,
    /// KONNWEI brand adapters
    KONNWEI,
    /// Veepeak brand adapters
    Veepeak,
    /// Foseal brand adapters
    Foseal,
    /// Panlong brand adapters
    Panlong,
}

impl ObdVendor {
    /// Get all vendor variants as an array
    /// Ordered from most specific to least specific to avoid false matches
    pub const fn all() -> [ObdVendor; 10] {
        use ObdVendor::*;
        // More specific patterns first (OBDLink before OBDII, OBDII before OBD)
        [OBDLink, OBDII, KONNWEI, Veepeak, Foseal, Panlong, Vgate, ICar, ELM, OBD]
    }

    /// Get the display name (original mixed case)
    pub const fn display_name(&self) -> &'static str {
        match self {
            ObdVendor::OBD => "OBD",
            ObdVendor::ELM => "ELM",
            ObdVendor::OBDII => "OBDII",
            ObdVendor::OBDLink => "OBDLink",
            ObdVendor::Vgate => "Vgate",
            ObdVendor::ICar => "iCar",
            ObdVendor::KONNWEI => "KONNWEI",
            ObdVendor::Veepeak => "Veepeak",
            ObdVendor::Foseal => "Foseal",
            ObdVendor::Panlong => "Panlong",
        }
    }

    /// Get the uppercase search pattern
    pub const fn search_pattern(&self) -> &'static str {
        match self {
            ObdVendor::OBD => "OBD",
            ObdVendor::ELM => "ELM",
            ObdVendor::OBDII => "OBDII",
            ObdVendor::OBDLink => "OBDLINK",
            ObdVendor::Vgate => "VGATE",
            ObdVendor::ICar => "ICAR",
            ObdVendor::KONNWEI => "KONNWEI",
            ObdVendor::Veepeak => "VEEPEAK",
            ObdVendor::Foseal => "FOSEAL",
            ObdVendor::Panlong => "PANLONG",
        }
    }

    /// Check if a device name contains this vendor pattern
    pub fn matches(&self, device_name: &str) -> bool {
        device_name.to_uppercase().contains(self.search_pattern())
    }

    /// Find vendor from device name (returns first match)
    pub fn from_device_name(device_name: &str) -> Option<ObdVendor> {
        ObdVendor::all()
            .iter()
            .find(|v| v.matches(device_name))
            .copied()
    }
}

impl fmt::Display for ObdVendor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl FromStr for ObdVendor {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s_upper = s.to_uppercase();
        ObdVendor::all()
            .iter()
            .find(|v| v.search_pattern() == s_upper)
            .copied()
            .ok_or_else(|| format!("Unknown OBD vendor: {}", s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_vendors() {
        let vendors = ObdVendor::all();
        assert_eq!(vendors.len(), 10);
    }

    #[test]
    fn test_matches() {
        assert!(ObdVendor::OBDLink.matches("OBDLink LX"));
        assert!(ObdVendor::ELM.matches("ELM327 v2.1"));
        assert!(ObdVendor::Vgate.matches("vgate icar pro"));
        assert!(!ObdVendor::OBDLink.matches("Some Random Device"));
    }

    #[test]
    fn test_from_device_name() {
        assert_eq!(
            ObdVendor::from_device_name("OBDLink LX"),
            Some(ObdVendor::OBDLink)
        );
        assert_eq!(
            ObdVendor::from_device_name("ELM327 v2.1"),
            Some(ObdVendor::ELM)
        );
        assert_eq!(ObdVendor::from_device_name("Unknown Device"), None);
    }

    #[test]
    fn test_display() {
        assert_eq!(ObdVendor::OBDLink.to_string(), "OBDLink");
        assert_eq!(ObdVendor::ICar.to_string(), "iCar");
        assert_eq!(ObdVendor::Vgate.to_string(), "Vgate");
    }

    #[test]
    fn test_from_str() {
        assert_eq!("OBDLINK".parse::<ObdVendor>(), Ok(ObdVendor::OBDLink));
        assert_eq!("elm".parse::<ObdVendor>(), Ok(ObdVendor::ELM));
        assert!("unknown".parse::<ObdVendor>().is_err());
    }
}
