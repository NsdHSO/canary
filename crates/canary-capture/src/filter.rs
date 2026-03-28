use canary_hardware::CanFrame;

/// CAN frame filter for selective capture
///
/// Supports filtering by:
/// - Single CAN ID
/// - CAN ID range
/// - Multiple CAN IDs (whitelist)
/// - Custom predicate function
#[derive(Clone)]
pub struct CanFilter {
    filter_type: FilterType,
}

#[derive(Clone)]
enum FilterType {
    /// Accept all frames
    AcceptAll,
    /// Accept specific CAN ID
    SingleId(u32),
    /// Accept CAN ID range (inclusive)
    IdRange { start: u32, end: u32 },
    /// Accept specific CAN IDs
    IdWhitelist(Vec<u32>),
    /// Accept by ID mask (id & mask == expected)
    IdMask { mask: u32, expected: u32 },
}

impl CanFilter {
    /// Create a filter that accepts all frames
    pub fn accept_all() -> Self {
        Self {
            filter_type: FilterType::AcceptAll,
        }
    }

    /// Create a filter for a single CAN ID
    pub fn single_id(id: u32) -> Self {
        Self {
            filter_type: FilterType::SingleId(id),
        }
    }

    /// Create a filter for a CAN ID range (inclusive)
    pub fn id_range(start: u32, end: u32) -> Self {
        Self {
            filter_type: FilterType::IdRange { start, end },
        }
    }

    /// Create a filter for specific CAN IDs
    pub fn id_whitelist(ids: Vec<u32>) -> Self {
        Self {
            filter_type: FilterType::IdWhitelist(ids),
        }
    }

    /// Create a filter using ID mask matching
    ///
    /// Matches when: (frame.id & mask) == expected
    pub fn id_mask(mask: u32, expected: u32) -> Self {
        Self {
            filter_type: FilterType::IdMask { mask, expected },
        }
    }

    /// Create a filter for UDS diagnostic frames (0x7E0-0x7EF)
    pub fn uds_diagnostic() -> Self {
        Self::id_range(0x7E0, 0x7EF)
    }

    /// Create a filter for OBD-II frames (0x7DF broadcast + 0x7E8-0x7EF responses)
    pub fn obd2() -> Self {
        Self::id_whitelist(vec![
            0x7DF, // OBD-II broadcast
            0x7E0, 0x7E1, 0x7E2, 0x7E3, // Request IDs
            0x7E4, 0x7E5, 0x7E6, 0x7E7, // Request IDs
            0x7E8, 0x7E9, 0x7EA, 0x7EB, // Response IDs
            0x7EC, 0x7ED, 0x7EE, 0x7EF, // Response IDs
        ])
    }

    /// Check if a frame matches this filter
    pub fn matches(&self, frame: &CanFrame) -> bool {
        match &self.filter_type {
            FilterType::AcceptAll => true,
            FilterType::SingleId(id) => frame.id == *id,
            FilterType::IdRange { start, end } => frame.id >= *start && frame.id <= *end,
            FilterType::IdWhitelist(ids) => ids.contains(&frame.id),
            FilterType::IdMask { mask, expected } => (frame.id & mask) == *expected,
        }
    }

    /// Get a human-readable description of this filter
    pub fn description(&self) -> String {
        match &self.filter_type {
            FilterType::AcceptAll => "Accept all".to_string(),
            FilterType::SingleId(id) => format!("ID 0x{:03X}", id),
            FilterType::IdRange { start, end } => {
                format!("ID range 0x{:03X}-0x{:03X}", start, end)
            }
            FilterType::IdWhitelist(ids) => {
                let ids_str: Vec<String> = ids.iter().map(|id| format!("0x{:03X}", id)).collect();
                format!("IDs [{}]", ids_str.join(", "))
            }
            FilterType::IdMask { mask, expected } => {
                format!("Mask 0x{:03X} == 0x{:03X}", mask, expected)
            }
        }
    }
}

impl Default for CanFilter {
    fn default() -> Self {
        Self::accept_all()
    }
}

impl std::fmt::Debug for CanFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CanFilter({})", self.description())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_frame(id: u32) -> CanFrame {
        CanFrame::new(id, vec![0x00])
    }

    #[test]
    fn test_accept_all() {
        let filter = CanFilter::accept_all();
        assert!(filter.matches(&make_frame(0x000)));
        assert!(filter.matches(&make_frame(0x7FF)));
        assert!(filter.matches(&make_frame(0x123)));
    }

    #[test]
    fn test_single_id() {
        let filter = CanFilter::single_id(0x7E0);
        assert!(filter.matches(&make_frame(0x7E0)));
        assert!(!filter.matches(&make_frame(0x7E1)));
        assert!(!filter.matches(&make_frame(0x000)));
    }

    #[test]
    fn test_id_range() {
        let filter = CanFilter::id_range(0x700, 0x7FF);
        assert!(filter.matches(&make_frame(0x700)));
        assert!(filter.matches(&make_frame(0x7FF)));
        assert!(filter.matches(&make_frame(0x750)));
        assert!(!filter.matches(&make_frame(0x6FF)));
        assert!(!filter.matches(&make_frame(0x800)));
    }

    #[test]
    fn test_id_whitelist() {
        let filter = CanFilter::id_whitelist(vec![0x7E0, 0x7E8, 0x7DF]);
        assert!(filter.matches(&make_frame(0x7E0)));
        assert!(filter.matches(&make_frame(0x7E8)));
        assert!(filter.matches(&make_frame(0x7DF)));
        assert!(!filter.matches(&make_frame(0x7E1)));
    }

    #[test]
    fn test_id_mask() {
        // Match all IDs where bits 7-4 are 0x7E (0x7E0-0x7EF)
        let filter = CanFilter::id_mask(0x7F0, 0x7E0);
        assert!(filter.matches(&make_frame(0x7E0)));
        assert!(filter.matches(&make_frame(0x7EF)));
        assert!(!filter.matches(&make_frame(0x7F0)));
        assert!(!filter.matches(&make_frame(0x7D0)));
    }

    #[test]
    fn test_uds_diagnostic_filter() {
        let filter = CanFilter::uds_diagnostic();
        assert!(filter.matches(&make_frame(0x7E0)));
        assert!(filter.matches(&make_frame(0x7E8)));
        assert!(filter.matches(&make_frame(0x7EF)));
        assert!(!filter.matches(&make_frame(0x7DF)));
        assert!(!filter.matches(&make_frame(0x100)));
    }

    #[test]
    fn test_obd2_filter() {
        let filter = CanFilter::obd2();
        assert!(filter.matches(&make_frame(0x7DF)));
        assert!(filter.matches(&make_frame(0x7E0)));
        assert!(filter.matches(&make_frame(0x7E8)));
        assert!(!filter.matches(&make_frame(0x100)));
    }

    #[test]
    fn test_default_is_accept_all() {
        let filter = CanFilter::default();
        assert!(filter.matches(&make_frame(0x123)));
    }

    #[test]
    fn test_filter_description() {
        assert_eq!(CanFilter::accept_all().description(), "Accept all");
        assert!(CanFilter::single_id(0x7E0).description().contains("7E0"));
        assert!(CanFilter::id_range(0x700, 0x7FF)
            .description()
            .contains("700"));
    }

    #[test]
    fn test_filter_debug() {
        let filter = CanFilter::single_id(0x7E0);
        let debug = format!("{:?}", filter);
        assert!(debug.contains("CanFilter"));
    }
}
