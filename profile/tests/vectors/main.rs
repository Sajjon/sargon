use profile::prelude::*;
use serde::Deserialize;
use std::{
    env,
    ffi::{OsStr, OsString},
    fs,
    path::PathBuf,
    str::FromStr,
};
use thiserror::Error;

fn crate_dir() -> PathBuf {
    env::var("CARGO_MANIFEST_DIR").unwrap().try_into().unwrap()
}

fn append_to_path(p: impl Into<OsString>, s: impl AsRef<OsStr>) -> PathBuf {
    let mut p = p.into();
    p.push(s);
    p.into()
}

#[derive(Debug, Error)]
pub enum TestingError {
    #[error("Failed to open file at path '{0}'")]
    FailedToOpenFile(PathBuf),

    #[error("File contents is not valid JSON '{0}'")]
    FailedDoesNotContainValidJSON(String),

    #[error("Failed to JSON deserialize string")]
    FailedToDeserialize(serde_json::Error),
}

/// `name` is file name without extension, assuming it is json file
#[cfg(not(tarpaulin_include))]
fn vector<'a, T>(name: impl AsRef<OsStr>) -> Result<T, TestingError>
where
    T: for<'de> Deserialize<'de>,
{
    let base = append_to_path(crate_dir(), "/tests/vectors/fixtures/");
    let base_file_path = append_to_path(base, name);
    let path = append_to_path(base_file_path, ".json");
    fs::read_to_string(path.clone())
        .map_err(|_| TestingError::FailedToOpenFile(path))
        .and_then(|j| {
            serde_json::Value::from_str(j.as_str())
                .map_err(|_| TestingError::FailedDoesNotContainValidJSON(j))
        })
        .and_then(|v| {
            serde_json::from_value::<T>(v)
                .map_err(|e| TestingError::FailedToDeserialize(e))
        })
}

#[test]
fn v100_100() {
    let profile =
        vector::<Profile>("only_plaintext_profile_snapshot_version_100")
            .expect("V100 Profile to deserialize");
    assert_eq!(
        profile.header.snapshot_version,
        ProfileSnapshotVersion::V100
    );
}

#[derive(Deserialize, Debug)]
struct CAP26Vector {
    path: HDPath,

    #[serde(rename = "publicKey")]
    public_key_hex: String,

    #[serde(rename = "entityKind")]
    entity_kind: CAP26EntityKind,

    #[serde(rename = "privateKey")]
    private_key_hex: String,

    #[serde(rename = "entityIndex")]
    entity_index: HDPathValue,

    #[serde(rename = "keyKind")]
    key_kind: CAP26KeyKind,
}

#[derive(Deserialize, Debug)]
struct CAP26Group {
    mnemonic: Mnemonic,
    tests: Vec<CAP26Vector>,
}
impl CAP26Group {
    fn test<S, P>(&self) -> ()
    where
        P: IsPublicKey<S::Signature>
            + FromStr<Err = CommonError>
            + std::fmt::Debug
            + PartialEq,
        S: IsPrivateKey<P> + FromStr<Err = CommonError> + std::fmt::Debug,
    {
        let seed = self.mnemonic.to_seed("");
        self.tests
            .iter()
            .map(|v| {
                let private_key = v.private_key::<S, P>()?;
                let derived: profile::PrivateKey = match S::curve() {
                    profile::SLIP10Curve::Curve25519 => {
                        MnemonicWithPassphrase::derive_ed25519_private_key(
                            &seed, &v.path,
                        )
                        .into()
                    }
                    profile::SLIP10Curve::Secp256k1 => {
                        MnemonicWithPassphrase::derive_secp256k1_private_key(
                            &seed, &v.path,
                        )
                        .into()
                    }
                };
                assert_eq!(derived.to_hex(), format!("{:?}", private_key));
                Ok::<(), CommonError>(())
            })
            .collect::<Result<Vec<()>, CommonError>>()
            .expect("All test vectors to pass");
    }
}

impl CAP26Vector {
    fn private_key<S, P>(&self) -> Result<S, CommonError>
    where
        P: IsPublicKey<S::Signature>
            + FromStr<Err = CommonError>
            + std::fmt::Debug
            + PartialEq,
        S: IsPrivateKey<P> + FromStr<Err = CommonError>,
    {
        let s: S = self.private_key_hex.parse()?;
        let p: P = self.public_key_hex.parse()?;
        assert_eq!(s.public_key(), p);
        Ok(s)
    }
}

#[test]
fn cap26() {
    let secp256k1 = vector::<CAP26Group>("cap26_secp256k1")
        .expect("CAP26 Secp256k1 vectors");
    let curve25519 = vector::<CAP26Group>("cap26_curve25519")
        .expect("CAP26 Curve25519 vectors");

    secp256k1.test::<Secp256k1PrivateKey, Secp256k1PublicKey>();
    curve25519.test::<Ed25519PrivateKey, Ed25519PublicKey>();
}
