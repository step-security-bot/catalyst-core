use super::*;
use crate::chaintypes::ChainLength;
use crate::header::{BftProof, BftSignature, Common, GenesisPraosProof, KesSignature};
use crate::key::BftLeaderId;
#[cfg(test)]
use crate::testing::serialization::serialization_bijection;
use chain_crypto::{
    self, AsymmetricKey, Ed25519, RistrettoGroup2HashDh, SecretKey, SumEd25519_12,
    VerifiableRandomFunction,
};
use lazy_static::lazy_static;
use quickcheck::{Arbitrary, Gen};
use test_strategy::proptest;

#[proptest]
fn header_serialization_bijection(b: Header) {
    serialization_bijection(b);
}

impl Arbitrary for BlockVersion {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        BlockVersion::from_u8(u8::arbitrary(g) % 3).unwrap()
    }
}

impl Arbitrary for AnyBlockVersion {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        u8::arbitrary(g).into()
    }
}

impl Arbitrary for Common {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        Common {
            block_version: Arbitrary::arbitrary(g),
            block_date: Arbitrary::arbitrary(g),
            block_content_size: Arbitrary::arbitrary(g),
            block_content_hash: Arbitrary::arbitrary(g),
            block_parent_hash: Arbitrary::arbitrary(g),
            chain_length: ChainLength(Arbitrary::arbitrary(g)),
        }
    }
}

impl Arbitrary for BftProof {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let sk: chain_crypto::SecretKey<Ed25519> = Arbitrary::arbitrary(g);
        let pk = sk.to_public();
        let signature = sk.sign(&[0u8, 1, 2, 3]);
        BftProof {
            leader_id: BftLeaderId(pk),
            signature: BftSignature(signature.coerce()),
        }
    }
}
impl Arbitrary for GenesisPraosProof {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        use chain_crypto::testing;
        let tcg = testing::TestCryptoGen::arbitrary(g);

        let node_id = Arbitrary::arbitrary(g);

        let vrf_proof = {
            let sk = RistrettoGroup2HashDh::generate(&mut tcg.get_rng(0));
            RistrettoGroup2HashDh::evaluate_and_prove(&sk, &[0, 1, 2, 3], &mut tcg.get_rng(1))
        };

        let kes_proof = {
            lazy_static! {
                static ref SK_FIRST: SecretKey<SumEd25519_12> = testing::static_secret_key();
            }
            let sk = SK_FIRST.clone();
            let signature = sk.sign(&[0u8, 1, 2, 3]);
            KesSignature(signature.coerce())
        };
        GenesisPraosProof {
            node_id,
            vrf_proof: vrf_proof.into(),
            kes_proof,
        }
    }
}

impl Arbitrary for Header {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let common = Common::arbitrary(g);
        let hdrbuilder = HeaderBuilderNew::new_raw(
            common.block_version,
            &common.block_content_hash,
            common.block_content_size,
        )
        .set_parent(&common.block_parent_hash, common.chain_length)
        .set_date(common.block_date);
        match common.block_version {
            BlockVersion::Genesis => hdrbuilder.into_unsigned_header().unwrap().generalize(),
            BlockVersion::Ed25519Signed => {
                let bft_proof: BftProof = Arbitrary::arbitrary(g);
                hdrbuilder
                    .into_bft_builder()
                    .unwrap()
                    .set_consensus_data(&bft_proof.leader_id)
                    .set_signature(bft_proof.signature)
                    .generalize()
            }
            BlockVersion::KesVrfproof => {
                let gp_proof: GenesisPraosProof = Arbitrary::arbitrary(g);
                hdrbuilder
                    .into_genesis_praos_builder()
                    .unwrap()
                    .set_consensus_data(&gp_proof.node_id, &gp_proof.vrf_proof)
                    .set_signature(gp_proof.kes_proof)
                    .generalize()
            }
        }
    }
}

mod proptest_impls {
    use chain_crypto::{
        digest::DigestOf,
        testing::{static_secret_key, TestCryptoGen},
        AsymmetricKey, Blake2b256, Ed25519, RistrettoGroup2HashDh, SecretKey, SumEd25519_12,
        VerifiableRandomFunction,
    };
    use proptest::{arbitrary::StrategyFor, prelude::*, strategy::Map};

    use crate::{
        block::{BftProof, BftSignature, BlockVersion, Common, GenesisPraosProof, KesSignature},
        certificate::PoolRegistration,
        header::{Header, HeaderBuilderNew},
        key::BftLeaderId,
    };

    impl Arbitrary for Header {
        type Parameters = ();
        type Strategy = Map<
            StrategyFor<(Common, BftProof, GenesisPraosProof)>,
            fn((Common, BftProof, GenesisPraosProof)) -> Self,
        >;

        fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
            any::<(Common, BftProof, GenesisPraosProof)>().prop_map(
                |(common, bft_proof, gp_proof)| {
                    let builder = HeaderBuilderNew::new_raw(
                        common.block_version,
                        &common.block_content_hash,
                        common.block_content_size,
                    )
                    .set_parent(&common.block_parent_hash, common.chain_length)
                    .set_date(common.block_date);

                    match common.block_version {
                        BlockVersion::Genesis => {
                            builder.into_unsigned_header().unwrap().generalize()
                        }
                        BlockVersion::Ed25519Signed => builder
                            .into_bft_builder()
                            .unwrap()
                            .set_consensus_data(&bft_proof.leader_id)
                            .set_signature(bft_proof.signature)
                            .generalize(),
                        BlockVersion::KesVrfproof => builder
                            .into_genesis_praos_builder()
                            .unwrap()
                            .set_consensus_data(&gp_proof.node_id, &gp_proof.vrf_proof)
                            .set_signature(gp_proof.kes_proof)
                            .generalize(),
                    }
                },
            )
        }
    }

    impl Arbitrary for BftProof {
        type Parameters = ();
        type Strategy = Map<StrategyFor<SecretKey<Ed25519>>, fn(SecretKey<Ed25519>) -> Self>;
        fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
            any::<SecretKey<Ed25519>>().prop_map(|sk| {
                let pk = sk.to_public();
                let signature = sk.sign(&[0u8, 1, 2, 3]);
                BftProof {
                    leader_id: BftLeaderId(pk),
                    signature: BftSignature(signature.coerce()),
                }
            })
        }
    }

    impl Arbitrary for GenesisPraosProof {
        type Parameters = ();
        type Strategy = Map<
            StrategyFor<(TestCryptoGen, DigestOf<Blake2b256, PoolRegistration>)>,
            fn((TestCryptoGen, DigestOf<Blake2b256, PoolRegistration>)) -> Self,
        >;

        fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
            any::<(TestCryptoGen, DigestOf<Blake2b256, PoolRegistration>)>().prop_map(
                |(tcg, node_id)| {
                    let vrf_proof = {
                        let sk = RistrettoGroup2HashDh::generate(&mut tcg.get_rng(0));
                        RistrettoGroup2HashDh::evaluate_and_prove(
                            &sk,
                            &[0, 1, 2, 3],
                            &mut tcg.get_rng(1),
                        )
                    };

                    let kes_proof = {
                        let sk: SecretKey<SumEd25519_12> = static_secret_key();
                        let signature = sk.sign(&[0u8, 1, 2, 3]);
                        KesSignature(signature.coerce())
                    };
                    Self {
                        node_id,
                        vrf_proof: vrf_proof.into(),
                        kes_proof,
                    }
                },
            )
        }
    }
}
