use super::{entity_address::EntityAddress, entity_type::EntityType, network_id::NetworkID};
use crate::utils::string_utils::suffix_string;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AccountAddress {
    pub address: String,
    pub network_id: NetworkID,
}

impl AccountAddress {
    pub fn short(&self) -> String {
        let suffix = suffix_string(6, &self.address);
        format!("{}...{}", &self.address[0..4], suffix)
    }
}

impl EntityAddress for AccountAddress {
    fn entity_type() -> EntityType {
        EntityType::Account
    }

    fn with_address_and_network_id(address: &str, network_id: NetworkID) -> Self {
        Self::validate(address);
        return Self {
            address: address.to_string(),
            network_id,
        };
    }
}

impl Display for AccountAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.address)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use radix_engine_common::crypto::{Ed25519PublicKey, PublicKey};

    use crate::{
        error::Error,
        v100::networks::{
            account_address::AccountAddress, entity_address::EntityAddress, network_id::NetworkID,
        },
    };

    #[test]
    fn from_bech32() {
        assert!(AccountAddress::try_from_bech32(
            "account_rdx16xlfcpp0vf7e3gqnswv8j9k58n6rjccu58vvspmdva22kf3aplease",
        )
        .is_ok());
    }

    #[test]
    fn from_public_key_bytes_and_network_id() {
        let public_key = Ed25519PublicKey::from_str(
            "3e9b96a2a863f1be4658ea66aa0584d2a8847d4c0f658b20e62e3594d994d73d",
        )
        .unwrap();
        assert_eq!(
            AccountAddress::from_public_key(PublicKey::Ed25519(public_key), NetworkID::Mainnet)
                .address,
            "account_rdx129qdd2yp9vs8jkkn2uwn6sw0ejwmcwr3r4c3usr2hp0nau67m2kzdm"
        )
    }

    #[test]
    fn network_id() {
        let sut = AccountAddress::try_from_bech32(
            "account_rdx16xlfcpp0vf7e3gqnswv8j9k58n6rjccu58vvspmdva22kf3aplease",
        )
        .unwrap();
        assert_eq!(sut.network_id, NetworkID::Mainnet);
    }

    #[test]
    fn short() {
        let sut: AccountAddress = AccountAddress::try_from_bech32(
            "account_rdx16xlfcpp0vf7e3gqnswv8j9k58n6rjccu58vvspmdva22kf3aplease",
        )
        .unwrap();
        assert_eq!(sut.short(), "acco...please");
    }

    #[test]
    fn invalid() {
        assert_eq!(
            AccountAddress::try_from_bech32("x"),
            Err(Error::FailedToDecodeAddressFromBech32)
        )
    }

    #[test]
    fn invalid_checksum() {
        assert_eq!(
            AccountAddress::try_from_bech32(
                "account_rdx16xlfcpp0vf7e3gqnswv8j9k58n6rjccu58vvspmdva22kf3apleasx"
            ),
            Err(Error::FailedToDecodeAddressFromBech32)
        )
    }

    #[test]
    fn invalid_entity_type() {
        assert_eq!(
            AccountAddress::try_from_bech32(
                "identity_rdx16xlfcpp0vf7e3gqnswv8j9k58n6rjccu58vvspmdva22kf3aplease"
            ),
            Err(Error::FailedToDecodeAddressFromBech32)
        )
    }
}
