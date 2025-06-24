use core::str::FromStr;
use revm::primitives::hardfork::{SpecId, UnknownHardfork};

#[repr(u8)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(non_camel_case_types)]
pub enum TaikoSpecId {
    /// Katla hard fork for the Taiko network
    KATLA,
    /// Hekla hard fork for the Taiko network
    HEKLA,
    /// Ontake hard fork for the Taiko network
    ONTAKE,
    /// Pacaya hard fork for the Taiko network
    PACAYA,
}

impl TaikoSpecId {
    /// Converts the [`TaikoSpecId`] into a [`SpecId`].
    pub const fn into_eth_spec(self) -> SpecId {
        match self {
            Self::KATLA | Self::HEKLA | Self::ONTAKE | Self::PACAYA => SpecId::SHANGHAI,
        }
    }

    pub const fn is_enabled_in(self, other: TaikoSpecId) -> bool {
        other as u8 <= self as u8
    }
}

impl From<TaikoSpecId> for SpecId {
    fn from(spec: TaikoSpecId) -> Self {
        spec.into_eth_spec()
    }
}

impl FromStr for TaikoSpecId {
    type Err = UnknownHardfork;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            name::KATLA => Ok(TaikoSpecId::KATLA),
            name::HEKLA => Ok(TaikoSpecId::HEKLA),
            name::ONTAKE => Ok(TaikoSpecId::ONTAKE),
            name::PACAYA => Ok(TaikoSpecId::PACAYA),
            _ => Err(UnknownHardfork),
        }
    }
}

impl From<TaikoSpecId> for &'static str {
    fn from(spec_id: TaikoSpecId) -> Self {
        match spec_id {
            TaikoSpecId::KATLA => name::KATLA,
            TaikoSpecId::HEKLA => name::HEKLA,
            TaikoSpecId::ONTAKE => name::ONTAKE,
            TaikoSpecId::PACAYA => name::PACAYA,
        }
    }
}

/// String identifiers for Optimism hardforks
pub mod name {
    pub const KATLA: &str = "Katla";
    pub const HEKLA: &str = "Hekla";
    pub const ONTAKE: &str = "Ontake";
    pub const PACAYA: &str = "Pacaya";
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::vec;

    #[test]
    fn test_taiko_spec_id_eth_spec_compatibility() {
        // Define test cases: (OpSpecId, enabled in ETH specs, enabled in OP specs)
        let test_cases = [(
            TaikoSpecId::ONTAKE,
            vec![
                (SpecId::MERGE, true),
                (SpecId::SHANGHAI, true),
                (SpecId::CANCUN, false),
                (SpecId::default(), false),
            ],
            vec![
                (TaikoSpecId::KATLA, true),
                (TaikoSpecId::HEKLA, true),
                (TaikoSpecId::ONTAKE, true),
                (TaikoSpecId::PACAYA, false),
            ],
        )];

        for (taiko_spec, eth_tests, taiko_tests) in test_cases {
            // Test ETH spec compatibility
            for (eth_spec, expected) in eth_tests {
                assert_eq!(
                    taiko_spec.into_eth_spec().is_enabled_in(eth_spec),
                    expected,
                    "{:?} should {} be enabled in ETH {:?}",
                    taiko_spec,
                    if expected { "" } else { "not " },
                    eth_spec
                );
            }

            // Test OP spec compatibility
            for (other_taiko_spec, expected) in taiko_tests {
                assert_eq!(
                    taiko_spec.is_enabled_in(other_taiko_spec),
                    expected,
                    "{:?} should {} be enabled in OP {:?}",
                    taiko_spec,
                    if expected { "" } else { "not " },
                    other_taiko_spec
                );
            }
        }
    }
}
