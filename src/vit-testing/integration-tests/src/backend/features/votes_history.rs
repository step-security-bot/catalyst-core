use crate::common::{iapyx_from_secret_key, vitup_setup};
use assert_fs::TempDir;
use chain_impl_mockchain::vote::Choice;
use jormungandr_lib::crypto::hash::Hash;
use jormungandr_lib::interfaces::AccountVotes;
use jormungandr_lib::interfaces::FragmentStatus;
use jormungandr_testing_utils::testing::node::time;
use std::collections::HashMap;
use std::str::FromStr;
use valgrind::{Proposal, Protocol};
use vit_servicing_station_tests::common::data::ArbitraryValidVotingTemplateGenerator;
use vitup::builders::VitBackendSettingsBuilder;
use vitup::config::VoteBlockchainTime;
use vitup::config::{InitialEntry, Initials};
use vitup::scenario::network::setup_network;
const PIN: &str = "1234";
const ALICE: &str = "alice";

#[test]
pub fn votes_history_reflects_casted_votes() {
    let testing_directory = TempDir::new().unwrap().into_persistent();
    let endpoint = "127.0.0.1:8080";
    let version = "2.0";
    let batch_size = 1;
    let vote_timing = VoteBlockchainTime {
        vote_start: 0,
        tally_start: 1,
        tally_end: 2,
        slots_per_epoch: 30,
    };

    let mut quick_setup = VitBackendSettingsBuilder::new();
    quick_setup
        .initials(Initials(vec![InitialEntry::Wallet {
            name: ALICE.to_string(),
            funds: 10_000,
            pin: PIN.to_string(),
        }]))
        .vote_timing(vote_timing.into())
        .slot_duration_in_seconds(2)
        .proposals_count(300)
        .voting_power(31_000)
        .private(true);

    let mut template_generator = ArbitraryValidVotingTemplateGenerator::new();
    let (mut vit_controller, mut controller, vit_parameters, _fund_name) =
        vitup_setup(quick_setup, testing_directory.path().to_path_buf());

    let (nodes, vit_station, wallet_proxy) = setup_network(
        &mut controller,
        &mut vit_controller,
        vit_parameters,
        &mut template_generator,
        endpoint.to_string(),
        &Protocol::Http,
        version.to_owned(),
    )
    .unwrap();

    let secret = testing_directory.path().join("vit_backend/wallet_alice");
    let mut alice = iapyx_from_secret_key(secret, &wallet_proxy).unwrap();

    let proposals = alice.proposals().unwrap();
    let votes_data: Vec<(&Proposal, Choice)> = proposals
        .iter()
        .take(batch_size)
        .map(|proposal| (proposal, Choice::new(0)))
        .collect();

    let fragment_ids = alice
        .votes_batch(votes_data.clone())
        .unwrap()
        .iter()
        .map(|item| item.to_string())
        .collect();

    time::wait_for_epoch(1, nodes[0].rest());

    let fragment_statuses = nodes[0].rest().fragments_statuses(fragment_ids).unwrap();
    println!("{:?}", fragment_statuses);
    assert!(fragment_statuses
        .iter()
        .all(|(_, status)| matches!(status, FragmentStatus::InABlock { .. })));

    let account_votes: Vec<AccountVotes> = {
        let mut votes_registry: HashMap<Hash, Vec<i64>> = nodes[0]
            .rest()
            .vote_plan_statuses()
            .unwrap()
            .iter()
            .map(|vote_plan| (vote_plan.id, Vec::new()))
            .collect();

        for (proposal, _choice) in votes_data {
            let hash = Hash::from_str(&proposal.chain_voteplan_id).unwrap();
            if let Some(registry) = votes_registry.get_mut(&hash) {
                registry.push(proposal.chain_proposal_index)
            }
        }

        votes_registry
            .into_iter()
            .map(|(vote_plan_id, votes)| AccountVotes {
                vote_plan_id,
                votes: votes.into_iter().map(|x| x as u8).collect(),
            })
            .collect()
    };

    let votes_history = alice
        .votes_history()
        .unwrap()
        .expect("vote history is empty");

    assert_eq!(votes_history, account_votes);
    for account_vote in account_votes {
        let actual_ids = alice
            .vote_plan_history(account_vote.vote_plan_id)
            .unwrap_or_else(|_| {
                panic!(
                    "vote history is empty for vote_plan: {} ",
                    account_vote.vote_plan_id
                )
            });
        assert_eq!(actual_ids, Some(account_vote.votes.clone()));
    }

    vit_station.shutdown();
    wallet_proxy.shutdown();
    for mut node in nodes {
        node.shutdown().unwrap();
    }
    controller.finalize();
}