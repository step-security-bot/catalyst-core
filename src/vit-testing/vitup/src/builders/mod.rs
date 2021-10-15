mod helpers;
pub mod post_deployment;
mod reviews;
pub mod utils;

pub use helpers::{
    convert_to_blockchain_date, convert_to_human_date, generate_qr_and_hashes,
    VitVotePlanDefBuilder, WalletExtension,
};

use crate::builders::helpers::build_servicing_station_parameters;
use crate::config::DataGenerationConfig;
use crate::config::VitStartParameters;
use crate::config::VoteBlockchainTime;
use crate::config::VoteTime;
use crate::scenario::controller::VitController;
use crate::scenario::controller::VitControllerBuilder;
use crate::{config::Initials, Result};
use assert_fs::fixture::{ChildPath, PathChild};
use chain_impl_mockchain::value::Value;
use chrono::naive::NaiveDateTime;
use jormungandr_lib::interfaces::CommitteeIdDef;
use jormungandr_lib::interfaces::ConsensusLeaderId;
pub use jormungandr_lib::interfaces::Initial;
use jormungandr_lib::time::SecondsSinceUnixEpoch;
use jormungandr_scenario_tests::scenario::settings::Settings;
use jormungandr_scenario_tests::scenario::{
    ActiveSlotCoefficient, ConsensusVersion, ContextChaCha, Controller, KesUpdateSpeed, Milli,
    NumberOfSlotsPerEpoch, SlotDuration, Topology,
};
use jormungandr_testing_utils::testing::network::{Blockchain, Node, WalletTemplate};
use jormungandr_testing_utils::wallet::LinearFee;
pub use reviews::ReviewGenerator;
use std::collections::HashMap;
use valgrind::Protocol;
use vit_servicing_station_tests::common::data::ValidVotePlanParameters;

pub const LEADER_1: &str = "Leader1";
pub const LEADER_2: &str = "Leader2";
pub const LEADER_3: &str = "Leader3";
pub const WALLET_NODE: &str = "Wallet_Node";

use crate::config::VOTE_TIME_FORMAT as FORMAT;

#[derive(Clone)]
pub struct VitBackendSettingsBuilder {
    config: DataGenerationConfig,
    committee_wallet: String,
    title: String,
    //needed for load tests when we rely on secret keys instead of qrs
    skip_qr_generation: bool,
    block0_date: SecondsSinceUnixEpoch,
}

impl Default for VitBackendSettingsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl VitBackendSettingsBuilder {
    pub fn new() -> Self {
        Self {
            config: Default::default(),
            title: "vit_backend".to_owned(),
            committee_wallet: "committee_1".to_owned(),
            skip_qr_generation: false,
            block0_date: SecondsSinceUnixEpoch::now(),
        }
    }

    pub fn fees(&mut self, fees: LinearFee) {
        self.config.linear_fees = fees;
    }

    pub fn block0_date(&self) -> SecondsSinceUnixEpoch {
        self.block0_date
    }

    pub fn set_external_committees(&mut self, external_committees: Vec<CommitteeIdDef>) {
        self.config.committees = external_committees;
    }

    pub fn skip_qr_generation(&mut self) {
        self.skip_qr_generation = true;
    }

    pub fn parameters(&self) -> &VitStartParameters {
        &self.config.params
    }

    pub fn with_protocol(&mut self, protocol: Protocol) -> &mut Self {
        self.config.params.protocol = protocol;
        self
    }

    pub fn protocol(&self) -> &Protocol {
        &self.config.params.protocol
    }

    pub fn title(&self) -> String {
        self.title.clone()
    }

    pub fn initials(&mut self, initials: Initials) -> &mut Self {
        self.config.params.initials = initials;
        self
    }

    pub fn initials_count(&mut self, initials_count: usize, pin: &str) -> &mut Self {
        self.initials(Initials::new_above_threshold(
            initials_count,
            &pin.to_string(),
        ));
        self
    }

    pub fn extend_initials(&mut self, initials: Vec<Initial>) -> &mut Self {
        self.config.params.initials.extend_from_external(initials);
        self
    }

    pub fn slot_duration_in_seconds(&mut self, slot_duration: u8) -> &mut Self {
        self.config.params.slot_duration = slot_duration;
        self
    }
    pub fn vote_timing(&mut self, vote_timing: VoteTime) -> &mut Self {
        self.config.params.vote_time = vote_timing;
        self
    }

    pub fn version(&mut self, version: String) -> &mut Self {
        self.config.params.version = version;
        self
    }

    pub fn proposals_count(&mut self, proposals_count: u32) -> &mut Self {
        self.config.params.proposals = proposals_count;
        self
    }

    pub fn challenges_count(&mut self, challenges_count: usize) -> &mut Self {
        self.config.params.challenges = challenges_count;
        self
    }

    pub fn voting_power(&mut self, voting_power: u64) -> &mut Self {
        self.config.params.voting_power = voting_power;
        self
    }

    pub fn consensus_leaders_ids(
        &mut self,
        consensus_leaders_ids: Vec<ConsensusLeaderId>,
    ) -> &mut Self {
        self.config.consensus_leader_ids = consensus_leaders_ids;
        self
    }

    pub fn next_vote_timestamp(&mut self, next_vote_timestamp: Option<String>) -> &mut Self {
        if let Some(timestamp) = next_vote_timestamp {
            self.config.params.next_vote_start_time =
                Some(NaiveDateTime::parse_from_str(&timestamp, FORMAT).unwrap());
        }
        self
    }

    pub fn refresh_timestamp(&mut self, refresh_timestamp: Option<String>) -> &mut Self {
        if let Some(timestamp) = refresh_timestamp {
            self.config.params.refresh_time =
                Some(NaiveDateTime::parse_from_str(&timestamp, FORMAT).unwrap());
        }
        self
    }

    pub fn fund_name(&self) -> String {
        self.config.params.fund_name.to_string()
    }

    pub fn fund_id(&mut self, id: i32) -> &mut Self {
        self.config.params.fund_id = id;
        self
    }

    pub fn private(&mut self, private: bool) -> &mut Self {
        self.config.params.private = private;
        self
    }

    pub fn upload_parameters(&mut self, parameters: VitStartParameters) {
        self.config.params = parameters;
    }

    pub fn build_topology(&mut self) -> Topology {
        let topology = Topology::default();

        // Leader 1
        let leader_1 = Node::new(LEADER_1);
       
        // leader 2
        let leader_2 = Node::new(LEADER_2)
        .with_trusted_peer(LEADER_1);
       
        // leader 3
        let leader_3 = Node::new(LEADER_3)
        .with_trusted_peer(LEADER_1)
        .with_trusted_peer(LEADER_2);
       
        // passive
        let passive = Node::new(WALLET_NODE)
        .with_trusted_peer(LEADER_1)
        .with_trusted_peer(LEADER_2)
        .with_trusted_peer(LEADER_3);

        topology
            .with_node(leader_1)
            .with_node(leader_2)
            .with_node(leader_3)
            .with_node(passive)
    }

    pub fn blockchain_timing(&self) -> VoteBlockchainTime {
        convert_to_blockchain_date(&self.config.params, self.block0_date)
    }

    pub fn dump_qrs(
        &self,
        controller: &Controller,
        initials: &HashMap<WalletTemplate, String>,
        child: &ChildPath,
    ) -> Result<()> {
        let folder = child.child("qr-codes");
        std::fs::create_dir_all(folder.path())?;

        let wallets: Vec<(_, _)> = controller
            .wallets()
            .filter(|(_, x)| !x.template().alias().starts_with("committee"))
            .map(|(alias, _template)| {
                (
                    alias,
                    controller
                        .wallet(alias)
                        .unwrap_or_else(|_| panic!("cannot find wallet with alias '{}'", alias)),
                )
            })
            .collect();

        generate_qr_and_hashes(wallets, initials, &self.config.params, &folder)
    }

    pub fn build(
        &mut self,
        mut context: ContextChaCha,
    ) -> Result<(VitController, Controller, ValidVotePlanParameters, String)> {
        let mut builder = VitControllerBuilder::new(&self.title);

        let vote_blockchain_time =
            convert_to_blockchain_date(&self.config.params, self.block0_date);

        let mut blockchain = Blockchain::new(
            ConsensusVersion::Bft,
            NumberOfSlotsPerEpoch::new(vote_blockchain_time.slots_per_epoch)
                .expect("valid number of slots per epoch"),
            SlotDuration::new(self.config.params.slot_duration)
                .expect("valid slot duration in seconds"),
            KesUpdateSpeed::new(46800).expect("valid kes update speed in seconds"),
            ActiveSlotCoefficient::new(Milli::from_millis(700))
                .expect("active slot coefficient in millis"),
        );

        println!("building topology..");

        builder.set_topology(self.build_topology());
        blockchain.add_leader(LEADER_1);
        blockchain.add_leader(LEADER_2);
        blockchain.add_leader(LEADER_3);

        println!("building blockchain parameters..");

        blockchain.set_linear_fee(self.config.linear_fees);
        blockchain.set_tx_max_expiry_epochs(self.config.params.tx_max_expiry_epochs);
        blockchain.set_discrimination(chain_addr::Discrimination::Production);
        blockchain.set_block_content_max_size(self.config.params.block_content_max_size.into());
        blockchain.set_block0_date(self.block0_date);

        if !self.config.consensus_leader_ids.is_empty() {
            blockchain.set_external_consensus_leader_ids(self.config.consensus_leader_ids.clone());
        }

        if !self.config.committees.is_empty() {
            blockchain.set_external_committees(self.config.committees.clone());
        }

        let committee_wallet = WalletTemplate::new_account(
            self.committee_wallet.clone(),
            Value(1_000_000_000),
            blockchain.discrimination(),
        );
        blockchain.add_wallet(committee_wallet);
        blockchain.add_committee(self.committee_wallet.clone());

        let child = context.child_directory(self.title());

        println!("building initials..");

        let mut templates = HashMap::new();
        if self.config.params.initials.any() {
            blockchain.set_external_wallets(self.config.params.initials.external_templates());
            templates = self
                .config
                .params
                .initials
                .templates(self.config.params.voting_power, blockchain.discrimination());
            for (wallet, _) in templates.iter().filter(|(x, _)| *x.value() > Value::zero()) {
                blockchain.add_wallet(wallet.clone());
            }
        }
        println!("building voteplan..");

        VitVotePlanDefBuilder::new(vote_blockchain_time)
            .options(3)
            .off_chain_action()
            .split_by(255)
            .fund_name(self.fund_name())
            .with_committee(self.committee_wallet.clone())
            .with_parameters(self.config.params.clone())
            .build()
            .into_iter()
            .for_each(|vote_plan_def| blockchain.add_vote_plan(vote_plan_def.alias(),vote_plan_def.owner(),chain_impl_mockchain::certificate::VotePlan::from(vote_plan_def).into()));
        builder.set_blockchain(blockchain);
        builder.build_settings(&mut context);

        println!("building controllers..");

        let (vit_controller, controller) = builder.build_controllers(context)?;

        if !self.skip_qr_generation {
            self.dump_qrs(&controller, &templates, &child)?;
        }

        println!("dumping secret keys..");

        controller.settings().dump_private_vote_keys(child);

        println!("build servicing station static data..");

        let parameters = build_servicing_station_parameters(
            self.fund_name(),
            &self.config.params,
            controller.vote_plans(),
            controller.settings(),
        );
        Ok((
            vit_controller,
            controller,
            parameters,
            self.config.params.version.clone(),
        ))
    }

    pub fn print_report(&self) {
        let parameters = self.parameters();

        println!("Fund id: {}", parameters.fund_id);
        println!(
            "refresh timestamp\t(registration_snapshot_time):\t\t\t{:?}",
            parameters.refresh_time
        );

        let (vote_start_timestamp, tally_start_timestamp, tally_end_timestamp) =
            convert_to_human_date(parameters, self.block0_date);

        println!(
            "vote start timestamp\t(fund_start_time, chain_vote_start_time):\t{:?}",
            vote_start_timestamp
        );
        println!(
            "tally start timestamp\t(fund_end_time, chain_vote_end_time):\t\t{:?}",
            tally_start_timestamp
        );
        println!(
            "tally end timestamp:\t(chain_committee_end_time)\t\t\t{:?}",
            tally_end_timestamp
        );
        println!(
            "next vote start time\t(next_fund_start_time):\t\t\t\t{:?}",
            parameters.next_vote_start_time
        );
    }
}
