use crate::accounting::account;
use crate::key::{deserialize_public_key, serialize_public_key};
use crate::transaction::WitnessAccountData;
use chain_core::property::WriteError;
use chain_core::{
    packer::Codec,
    property::{DeserializeFromSlice, ReadError, Serialize},
};
use chain_crypto::{Ed25519, PublicKey, Signature};

pub use account::{DelegationRatio, DelegationType, LedgerError, SpendingCounter};

pub type AccountAlg = Ed25519;

pub type Witness = Signature<WitnessAccountData, AccountAlg>;

/// Account Identifier (also used as Public Key)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Identifier(PublicKey<AccountAlg>);

impl From<PublicKey<AccountAlg>> for Identifier {
    fn from(pk: PublicKey<AccountAlg>) -> Self {
        Identifier(pk)
    }
}

impl From<Identifier> for PublicKey<AccountAlg> {
    fn from(i: Identifier) -> Self {
        i.0
    }
}

impl AsRef<PublicKey<AccountAlg>> for Identifier {
    fn as_ref(&self) -> &PublicKey<AccountAlg> {
        &self.0
    }
}

impl Serialize for Identifier {
    fn serialized_size(&self) -> usize {
        self.0.as_ref().len()
    }

    fn serialize<W: std::io::Write>(&self, codec: &mut Codec<W>) -> Result<(), WriteError> {
        serialize_public_key(&self.0, codec)
    }
}

impl DeserializeFromSlice for Identifier {
    fn deserialize_from_slice(codec: &mut Codec<&[u8]>) -> Result<Self, ReadError> {
        deserialize_public_key(codec).map(Identifier)
    }
}

/// The public ledger of all accounts associated with their current state
pub type Ledger = account::Ledger<Identifier, ()>;

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::testing::serialization::serialization_bijection;
    use chain_crypto::{Ed25519, KeyPair};
    use quickcheck::{Arbitrary, Gen};
    use test_strategy::proptest;

    impl Arbitrary for Identifier {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let kp: KeyPair<Ed25519> = Arbitrary::arbitrary(g);
            Identifier::from(kp.into_keys().1)
        }
    }

    #[proptest]
    fn identifier_serialization_bijection(id: Identifier) {
        serialization_bijection(id);
    }
}

mod proptest_impl {
    use chain_crypto::{Ed25519, KeyPair};
    use proptest::{arbitrary::StrategyFor, prelude::*, strategy::Map};

    use super::Identifier;

    impl Arbitrary for Identifier {
        type Parameters = ();
        type Strategy = Map<StrategyFor<KeyPair<Ed25519>>, fn(KeyPair<Ed25519>) -> Self>;

        fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
            any::<KeyPair<Ed25519>>().prop_map(|kp| Identifier::from(kp.into_keys().1))
        }
    }
}
