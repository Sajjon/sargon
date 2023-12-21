#[cfg(any(test, feature = "placeholder"))]
use crate::v100::factors::factor_source_kind::FactorSourceKind;
use derive_getters::Getters;
#[cfg(any(test, feature = "placeholder"))]
use hd::{bip32::HDPathValue, derivation::mnemonic_with_passphrase::MnemonicWithPassphrase};
#[cfg(any(test, feature = "placeholder"))]
use wallet_kit_common::network_id::NetworkID;

use hd::{
    derivation::{
        derivation_path::DerivationPath,
        hierarchical_deterministic_public_key::HierarchicalDeterministicPublicKey,
    },
    AccountPath, CAP26KeyKind, CAP26Path, CAP26Repr,
};
use serde::{de, Deserializer, Serialize, Serializer};
use wallet_kit_common::types::keys::public_key::PublicKey;

use wallet_kit_common::error::common_error::CommonError as Error;

use super::{
    factor_instance::{
        factor_instance::FactorInstance, factor_instance_badge::FactorInstanceBadge,
    },
    factor_source_id::FactorSourceID,
    factor_source_id_from_hash::FactorSourceIDFromHash,
};

/// A virtual hierarchical deterministic `FactorInstance`
#[derive(Clone, Debug, PartialEq, Eq, Getters)]
pub struct HierarchicalDeterministicFactorInstance {
    factor_source_id: FactorSourceIDFromHash,
    public_key: HierarchicalDeterministicPublicKey,
}

impl HierarchicalDeterministicFactorInstance {
    pub fn derivation_path(&self) -> DerivationPath {
        self.public_key().derivation_path().clone()
    }

    pub fn new(
        factor_source_id: FactorSourceIDFromHash,
        public_key: HierarchicalDeterministicPublicKey,
    ) -> Self {
        Self {
            factor_source_id,
            public_key,
        }
    }

    pub fn with_key_and_path(
        factor_source_id: FactorSourceIDFromHash,
        public_key: PublicKey,
        derivation_path: DerivationPath,
    ) -> Self {
        Self::new(
            factor_source_id,
            HierarchicalDeterministicPublicKey::new(public_key, derivation_path),
        )
    }

    pub fn try_from(
        factor_source_id: FactorSourceID,
        public_key: PublicKey,
        derivation_path: DerivationPath,
    ) -> Result<Self, Error> {
        let factor_source_id = factor_source_id
            .as_hash()
            .ok_or(Error::FactorSourceIDNotFromHash)?;
        Ok(Self::with_key_and_path(
            factor_source_id.clone(),
            public_key,
            derivation_path,
        ))
    }

    pub fn try_from_factor_instance(factor_instance: FactorInstance) -> Result<Self, Error> {
        let virtual_source = factor_instance
            .badge()
            .as_virtual()
            .ok_or(Error::BadgeIsNotVirtualHierarchicalDeterministic)?;

        let badge = virtual_source.as_hierarchical_deterministic();

        Self::try_from(
            factor_instance.factor_source_id().clone(),
            badge.public_key().clone(),
            badge.derivation_path().clone(),
        )
    }

    pub fn factor_instance(&self) -> FactorInstance {
        FactorInstance::new(
            self.factor_source_id.clone().into(),
            FactorInstanceBadge::Virtual(self.public_key.clone().into()),
        )
    }

    pub fn key_kind(&self) -> Option<CAP26KeyKind> {
        match &self.derivation_path() {
            DerivationPath::CAP26(cap26) => match cap26 {
                CAP26Path::GetID(_) => None,
                CAP26Path::IdentityPath(identity_path) => Some(identity_path.key_kind().clone()),
                CAP26Path::AccountPath(account_path) => Some(account_path.key_kind().clone()),
            },
            DerivationPath::BIP44Like(_) => None,
        }
    }
}

impl Serialize for HierarchicalDeterministicFactorInstance {
    #[cfg(not(tarpaulin_include))] // false negative
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        self.factor_instance().serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for HierarchicalDeterministicFactorInstance {
    #[cfg(not(tarpaulin_include))] // false negative
    fn deserialize<D: Deserializer<'de>>(
        d: D,
    ) -> Result<HierarchicalDeterministicFactorInstance, D::Error> {
        FactorInstance::deserialize(d).and_then(|fi| {
            HierarchicalDeterministicFactorInstance::try_from_factor_instance(fi)
                .map_err(de::Error::custom)
        })
    }
}

#[cfg(any(test, feature = "placeholder"))]
impl HierarchicalDeterministicFactorInstance {
    /// A placeholder used to facilitate unit tests.
    pub fn placeholder() -> Self {
        Self::placeholder_transaction_signing()
    }

    /// A placeholder used to facilitate unit tests.
    pub fn placeholder_transaction_signing() -> Self {
        Self::placeholder_with_key_kind(CAP26KeyKind::TransactionSigning, 0)
    }

    /// A placeholder used to facilitate unit tests.
    pub fn placeholder_auth_signing() -> Self {
        Self::placeholder_with_key_kind(CAP26KeyKind::AuthenticationSigning, 0)
    }

    /// A placeholder used to facilitate unit tests.
    fn placeholder_with_key_kind(key_kind: CAP26KeyKind, index: HDPathValue) -> Self {
        let mwp = MnemonicWithPassphrase::placeholder();
        let path = AccountPath::new(NetworkID::Mainnet, key_kind, index);
        let private_key = mwp.derive_private_key(path.clone());
        let public_key = private_key.public_key();
        let id =
            FactorSourceIDFromHash::from_mnemonic_with_passphrase(FactorSourceKind::Device, mwp);
        Self::new(id.into(), public_key)
    }
}

#[cfg(test)]
mod tests {
    use hd::{
        bip44::bip44_like_path::BIP44LikePath,
        derivation::{
            derivation::Derivation, derivation_path::DerivationPath,
            hierarchical_deterministic_public_key::HierarchicalDeterministicPublicKey,
        },
        CAP26KeyKind, GetIDPath, IdentityPath,
    };
    use wallet_kit_common::{
        json::assert_eq_after_json_roundtrip, types::keys::public_key::PublicKey,
    };

    use crate::v100::factors::factor_source_id_from_hash::FactorSourceIDFromHash;

    use super::HierarchicalDeterministicFactorInstance;

    #[test]
    fn json_roundtrip() {
        let model = HierarchicalDeterministicFactorInstance::placeholder();
        assert_eq_after_json_roundtrip(
            &model,
            r#"
            {
				"badge": {
					"virtualSource": {
						"hierarchicalDeterministicPublicKey": {
							"publicKey": {
								"curve": "curve25519",
								"compressedData": "d24cc6af91c3f103d7f46e5691ce2af9fea7d90cfb89a89d5bba4b513b34be3b"
							},
							"derivationPath": {
								"scheme": "cap26",
								"path": "m/44H/1022H/1H/525H/1460H/0H"
							}
						},
						"discriminator": "hierarchicalDeterministicPublicKey"
					},
					"discriminator": "virtualSource"
				},
				"factorSourceID": {
					"fromHash": {
						"kind": "device",
						"body": "3c986ebf9dcd9167a97036d3b2c997433e85e6cc4e4422ad89269dac7bfea240"
					},
					"discriminator": "fromHash"
				}
			}
            "#,
        );
    }

    #[test]
    fn key_kind_bip44_is_none() {
        let derivation_path: DerivationPath = BIP44LikePath::placeholder().into();
        let sut = HierarchicalDeterministicFactorInstance::new(
            FactorSourceIDFromHash::placeholder(),
            HierarchicalDeterministicPublicKey::new(
                PublicKey::placeholder_ed25519(),
                derivation_path,
            ),
        );
        assert_eq!(sut.key_kind(), None);
    }

    #[test]
    fn key_kind_identity() {
        let derivation_path: DerivationPath = IdentityPath::placeholder().into();
        let sut = HierarchicalDeterministicFactorInstance::new(
            FactorSourceIDFromHash::placeholder(),
            HierarchicalDeterministicPublicKey::new(
                PublicKey::placeholder_ed25519(),
                derivation_path,
            ),
        );
        assert_eq!(sut.key_kind(), Some(CAP26KeyKind::TransactionSigning));
    }

    #[test]
    fn key_kind_cap26_getid_is_none() {
        let derivation_path: DerivationPath = GetIDPath::default().into();
        let sut = HierarchicalDeterministicFactorInstance::new(
            FactorSourceIDFromHash::placeholder(),
            HierarchicalDeterministicPublicKey::new(
                PublicKey::placeholder_ed25519(),
                derivation_path,
            ),
        );
        assert_eq!(sut.key_kind(), None);
    }

    #[test]
    fn placeholder_auth() {
        assert_eq!(
            HierarchicalDeterministicFactorInstance::placeholder_auth_signing()
                .derivation_path()
                .to_string(),
            "m/44H/1022H/1H/525H/1678H/0H"
        );
    }
}
