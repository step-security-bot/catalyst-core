use crate::common::{load::build_load_config, load::private_vote_test_scenario, vitup_setup};
use assert_fs::TempDir;
use iapyx::NodeLoad;
use jortestkit::measurement::Status;
use valgrind::Protocol;
use vit_servicing_station_tests::common::data::ArbitraryValidVotingTemplateGenerator;
use vitup::builders::VitBackendSettingsBuilder;
use vitup::config::VoteBlockchainTime;
use vitup::scenario::network::setup_network;

#[test]
pub fn load_test_public_100_000_votes() {
    let testing_directory = TempDir::new().unwrap().into_persistent();
    let endpoint = "127.0.0.1:8080";

    let version = "2.0";
    let no_of_threads = 10;
    let no_of_wallets = 40_000;
    let vote_timing = VoteBlockchainTime {
        vote_start: 0,
        tally_start: 100,
        tally_end: 102,
        slots_per_epoch: 60,
    };

    let mut quick_setup = VitBackendSettingsBuilder::new();
    quick_setup
        .initials_count(no_of_wallets, "1234")
        .vote_timing(vote_timing.into())
        .slot_duration_in_seconds(2)
        .proposals_count(300)
        .voting_power(31_000)
        .private(false);

    let setup_parameters = quick_setup.parameters().clone();
    let mut template_generator = ArbitraryValidVotingTemplateGenerator::new();
    let (mut vit_controller, mut controller, vit_parameters, fund_name) =
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

    let mut qr_codes_folder = testing_directory.path().to_path_buf();
    qr_codes_folder.push("vit_backend/qr-codes");

    let config = build_load_config(
        endpoint,
        qr_codes_folder,
        no_of_threads,
        100,
        1,
        setup_parameters,
    );
    let iapyx_load = NodeLoad::new(config);
    if let Some(benchmark) = iapyx_load.start().unwrap() {
        assert!(benchmark.status() == Status::Green, "too low efficiency");
    }

    vote_timing.wait_for_tally_start(nodes.get(0).unwrap().rest());

    let mut committee = controller.wallet("committee").unwrap();
    let vote_plan = controller.vote_plan(&fund_name).unwrap();

    controller
        .fragment_sender()
        .send_public_vote_tally(&mut committee, &vote_plan.into(), nodes.get(0).unwrap())
        .unwrap();

    vit_station.shutdown();
    wallet_proxy.shutdown();

    for mut node in nodes {
        node.logger()
            .assert_no_errors(&format!("Errors in logs for node: {}", node.alias()));
        node.shutdown().unwrap();
    }

    controller.finalize();
}

#[test]
pub fn load_test_private_pesimistic() {
    let no_of_threads = 10;
    let endpoint = "127.0.0.1:8080";
    let no_of_wallets = 8_000;
    let mut quick_setup = VitBackendSettingsBuilder::new();
    let vote_timing = VoteBlockchainTime {
        vote_start: 0,
        tally_start: 11,
        tally_end: 12,
        slots_per_epoch: 3,
    };

    quick_setup
        .initials_count(no_of_wallets, "1234")
        .vote_timing(vote_timing.into())
        .slot_duration_in_seconds(20)
        .proposals_count(250)
        .voting_power(31_000)
        .private(true);

    private_vote_test_scenario(quick_setup, endpoint, no_of_threads, 1);
}

#[test]
pub fn load_test_private_optimistic() {
    let no_of_threads = 10;
    let no_of_wallets = 20_000;
    let endpoint = "127.0.0.1:8080";
    let vote_timing = VoteBlockchainTime {
        vote_start: 6,
        tally_start: 11,
        tally_end: 12,
        slots_per_epoch: 180,
    };

    let mut quick_setup = VitBackendSettingsBuilder::new();
    quick_setup
        .initials_count(no_of_wallets, "1234")
        .vote_timing(vote_timing.into())
        .slot_duration_in_seconds(20)
        .proposals_count(500)
        .voting_power(31_000)
        .private(true);

    private_vote_test_scenario(quick_setup, endpoint, no_of_threads, 1);
}