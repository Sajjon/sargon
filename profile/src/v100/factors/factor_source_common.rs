use std::time::SystemTime;

use iso8601_timestamp::Timestamp;
use serde::{Deserialize, Serialize};

#[cfg(any(test, feature = "placeholder"))]
use crate::HasPlaceholder;
use crate::now;

use super::{
    factor_source_crypto_parameters::FactorSourceCryptoParameters,
    factor_source_flag::FactorSourceFlag,
};

/// Common properties shared between FactorSources of different kinds, describing
/// its state, when added, and supported cryptographic parameters.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct FactorSourceCommon {
    /// Cryptographic parameters a certain FactorSource supports, e.g. Elliptic Curves.
    ///
    /// Has interior mutability since Radix Wallet App version 1.3.0, it is
    /// possible to add crypto parameters to a FactorSource, e.g. when a user
    /// with a DeviceFactorSource with babylon crypto parameters, lets call it `B`,
    /// with mnemonic `M` adds `M` again but as an "Olympia" factor source, then
    /// the olympia crypto parameters are added to `B`.
    pub crypto_parameters: FactorSourceCryptoParameters,

    /// When this factor source for originally added by the user.
    pub added_on: SystemTime, // FIXME: NOW!

    /// Date of last usage of this factor source
    ///
    /// This is the only mutable property, it is mutable
    /// since we will update it every time this FactorSource
    /// is used.
    pub last_used_on: SystemTime, // FIXME: NOW!

    /// Flags which describe a certain state a FactorSource might be in, e.g. `Main` (BDFS).
    pub flags: Vec<FactorSourceFlag>, // FIXME: Change to Set
}

impl FactorSourceCommon {
    pub fn with_values<I>(
        crypto_parameters: FactorSourceCryptoParameters,
        added_on: Timestamp,
        last_used_on: Timestamp,
        flags: I,
    ) -> Self
    where
        I: IntoIterator<Item = FactorSourceFlag>,
    {
        Self {
            crypto_parameters,
            added_on: added_on.into(),
            last_used_on: last_used_on.into(),
            flags: Vec::from_iter(flags.into_iter()),
        }
    }

    pub fn new<I>(crypto_parameters: FactorSourceCryptoParameters, flags: I) -> Self
    where
        I: IntoIterator<Item = FactorSourceFlag>,
    {
        let date = now();
        Self::with_values(crypto_parameters, date, date, flags)
    }

    pub fn new_bdfs(is_main: bool) -> Self {
        Self::new(
            FactorSourceCryptoParameters::babylon(),
            if is_main {
                vec![FactorSourceFlag::Main]
            } else {
                Vec::new()
            }
            .into_iter(),
        )
    }
}

impl Default for FactorSourceCommon {
    fn default() -> Self {
        Self::new(FactorSourceCryptoParameters::default(), [])
    }
}

#[cfg(any(test, feature = "placeholder"))]
impl HasPlaceholder for FactorSourceCommon {
    /// A placeholder used to facilitate unit tests.
    fn placeholder() -> Self {
        Self::placeholder_main_babylon()
    }

    fn placeholder_other() -> Self {
        Self::placeholder_olympia()
    }
}

#[cfg(any(test, feature = "placeholder"))]
impl FactorSourceCommon {
    /// A placeholder used to facilitate unit tests.
    pub fn placeholder_main_babylon() -> Self {
        let date = Timestamp::parse("2023-09-11T16:05:56.000Z").unwrap();
        FactorSourceCommon::with_values(
            FactorSourceCryptoParameters::babylon(),
            date.clone(),
            date,
            [FactorSourceFlag::Main],
        )
    }

    /// A placeholder used to facilitate unit tests.
    pub fn placeholder_olympia() -> Self {
        let date = Timestamp::parse("2023-09-11T16:05:56.000Z").unwrap();
        FactorSourceCommon::with_values(
            FactorSourceCryptoParameters::olympia(),
            date.clone(),
            date,
            [],
        )
    }
}

#[cfg(test)]
mod tests {

    use crate::{assert_eq_after_json_roundtrip, HasPlaceholder};
    use iso8601_timestamp::Timestamp;

    use crate::v100::factors::{
        factor_source_crypto_parameters::FactorSourceCryptoParameters,
        factor_source_flag::FactorSourceFlag,
    };

    use super::FactorSourceCommon;

    #[test]
    fn equality() {
        assert_eq!(
            FactorSourceCommon::placeholder(),
            FactorSourceCommon::placeholder()
        );
        assert_eq!(
            FactorSourceCommon::placeholder_other(),
            FactorSourceCommon::placeholder_other()
        );
    }

    #[test]
    fn inequality() {
        assert_ne!(
            FactorSourceCommon::placeholder(),
            FactorSourceCommon::placeholder_other()
        );
    }

    #[test]
    fn default_support_babylon() {
        assert_eq!(
            FactorSourceCommon::default().crypto_parameters,
            FactorSourceCryptoParameters::babylon()
        )
    }

    // #[test]
    // fn new_uses_now_as_date() {
    //     let date0 = now();
    //     let model = FactorSourceCommon::new(FactorSourceCryptoParameters::default(), []);
    //     let mut date1 = now();
    //     for _ in 0..10 {
    //         // rust is too fast... lol.
    //         date1 = now();
    //     }
    //     let do_test = |d: Timestamp| {
    //         assert!(d > date0);
    //         assert!(d < date1);
    //     };
    //     do_test(model.added_on);
    //     do_test(model.last_used_on);
    // }

    #[test]
    fn json_roundtrip() {
        let date = Timestamp::parse("2023-09-11T16:05:56.000Z").unwrap();
        let model =
            FactorSourceCommon::with_values(
                FactorSourceCryptoParameters::default(),
                date.clone(),
                date,
                [FactorSourceFlag::Main],
            );

        assert_eq_after_json_roundtrip(
            &model,
            r#"
            {
                "addedOn": "2023-09-11T16:05:56.000Z",
                "cryptoParameters": {
                    "supportedCurves": ["curve25519"],
                    "supportedDerivationPathSchemes": ["cap26"]
                },
                "flags": ["main"],
                "lastUsedOn": "2023-09-11T16:05:56.000Z"
            }
            "#,
        );
    }

    #[test]
    fn main_flag_present_if_main() {
        assert!(FactorSourceCommon::new_bdfs(true)
            .flags
            .contains(&FactorSourceFlag::Main));
    }

    #[test]
    fn main_flag_not_present_if_not_main() {
        assert!(FactorSourceCommon::new_bdfs(false).flags.is_empty());
    }
}
