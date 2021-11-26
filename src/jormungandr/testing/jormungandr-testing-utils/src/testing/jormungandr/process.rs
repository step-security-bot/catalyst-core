use super::{starter::StartupError, JormungandrError};
use crate::testing::jcli::JCli;
use crate::testing::jormungandr::TestingDirectory;
use crate::testing::network::NodeAlias;
use crate::testing::node::grpc::JormungandrClient;
use crate::testing::{
    node::{
        uri_from_socket_addr, Explorer, JormungandrLogger, JormungandrRest,
        JormungandrStateVerifier, LogLevel,
    },
    utils, BlockDateGenerator, SyncNode, TestConfig,
};
use crate::testing::{
    FragmentChainSender, FragmentSender, FragmentSenderSetup, RemoteJormungandr,
    RemoteJormungandrBuilder,
};
use ::multiaddr::Multiaddr;
use chain_impl_mockchain::{block::BlockDate, fee::LinearFee};
use chain_time::TimeEra;
use jormungandr_lib::interfaces::NodeState;
use jormungandr_lib::{
    crypto::hash::Hash,
    interfaces::{Block0Configuration, TrustedPeer},
};
use jortestkit::prelude::NamedProcess;
use std::net::SocketAddr;
use std::path::Path;
use std::process::Child;
use std::process::ExitStatus;
use std::time::{Duration, Instant};
use thiserror::Error;

pub enum StartupVerificationMode {
    Log,
    Rest,
}

#[derive(PartialEq, Debug)]
pub enum Status {
    Running,
    Starting,
    Exited(ExitStatus),
}

impl From<NodeState> for Status {
    fn from(node_state: NodeState) -> Self {
        match &node_state {
            NodeState::Running => Status::Running,
            _ => Status::Starting,
        }
    }
}

pub struct JormungandrProcess {
    pub child: Child,
    pub logger: JormungandrLogger,
    grpc_client: JormungandrClient,
    temp_dir: Option<TestingDirectory>,
    alias: String,
    p2p_public_address: Multiaddr,
    p2p_listen_address: SocketAddr,
    rest_socket_addr: SocketAddr,
    block0_configuration: Block0Configuration,
}

impl JormungandrProcess {
    pub fn new<Conf: TestConfig>(
        mut child: Child,
        node_config: &Conf,
        block0_configuration: Block0Configuration,
        temp_dir: Option<TestingDirectory>,
        alias: String,
    ) -> Result<Self, StartupError> {
        let stdout = child.stdout.take().unwrap();

        Ok(JormungandrProcess {
            child,
            temp_dir,
            alias,
            grpc_client: JormungandrClient::new(node_config.p2p_listen_address()),
            logger: JormungandrLogger::new(stdout),
            p2p_public_address: node_config.p2p_public_address(),
            p2p_listen_address: node_config.p2p_listen_address(),
            rest_socket_addr: node_config.rest_socket_addr(),
            block0_configuration,
        })
    }

    pub fn process_id(&self) -> u32 {
        self.child.id()
    }

    pub fn grpc(&self) -> JormungandrClient {
        self.grpc_client.clone()
    }

    pub fn fragment_sender<'a, S: SyncNode + Send>(
        &self,
        setup: FragmentSenderSetup<'a, S>,
    ) -> FragmentSender<'a, S> {
        FragmentSender::new(
            self.genesis_block_hash(),
            self.fees(),
            self.default_block_date_generator(),
            setup,
        )
    }

    pub fn default_block_date_generator(&self) -> BlockDateGenerator {
        BlockDateGenerator::rolling_from_blockchain_config(
            &self.block0_configuration.blockchain_configuration,
            BlockDate {
                epoch: 1,
                slot_id: 0,
            },
            false,
        )
    }

    pub fn fragment_chain_sender<'a, S: SyncNode + Send>(
        &self,
        setup: FragmentSenderSetup<'a, S>,
    ) -> FragmentChainSender<'a, S> {
        FragmentChainSender::new(
            self.genesis_block_hash(),
            self.fees(),
            self.default_block_date_generator(),
            setup,
            self.to_remote(),
        )
    }

    pub fn wait_for_bootstrap(
        &self,
        verification_mode: &StartupVerificationMode,
        timeout: Duration,
    ) -> Result<(), StartupError> {
        let start = Instant::now();
        loop {
            if start.elapsed() > timeout {
                return Err(StartupError::Timeout {
                    timeout: timeout.as_secs(),
                    log_content: self.logger.get_log_content(),
                });
            }

            let stauts_result = self.status(verification_mode);

            if let Ok(status) = stauts_result {
                match status {
                    Status::Running => {
                        return Ok(());
                    }
                    Status::Exited(exit_status) => {
                        return Err(StartupError::ProcessExited(exit_status))
                    }
                    Status::Starting => (),
                }
            }
            std::thread::sleep(Duration::from_secs(2));
        }
    }

    pub fn wait_for_shutdown(
        &mut self,
        timeout: Duration,
    ) -> Result<Option<ExitStatus>, ShutdownError> {
        let start = Instant::now();
        loop {
            if start.elapsed() > timeout {
                return Err(ShutdownError::Timeout {
                    timeout: timeout.as_secs(),
                    log_content: self.logger.get_log_content(),
                });
            }
            if let Ok(maybe_exit_status) = self.child.try_wait() {
                return Ok(maybe_exit_status);
            }
            std::thread::sleep(Duration::from_secs(2));
        }
    }

    fn check_startup_errors_in_logs(&self) -> Result<(), JormungandrError> {
        let port_occupied_msgs = ["error 87", "error 98", "panicked at 'Box<Any>'"];
        if self.logger.contains_any_of(&port_occupied_msgs) {
            return Err(JormungandrError::PortAlreadyInUse);
        }

        self.check_no_errors_in_log()
    }

    pub fn status(&self, strategy: &StartupVerificationMode) -> Result<Status, StartupError> {
        match strategy {
            StartupVerificationMode::Log => {
                let bootstrap_completed_msgs = [
                    "listening and accepting gRPC connections",
                    "genesis block fetched",
                ];

                self.check_startup_errors_in_logs()?;

                if self.logger.contains_any_of(&bootstrap_completed_msgs) {
                    Ok(Status::Running)
                } else {
                    Ok(Status::Starting)
                }
            }
            StartupVerificationMode::Rest => {
                let output = self.rest().stats();
                if let Err(err) = output {
                    return Err(StartupError::CannotGetRestStatus(err));
                }

                match output.ok().as_ref() {
                    Some(node_stats) => Ok(node_stats.state.clone().into()),
                    _ => self
                        .check_startup_errors_in_logs()
                        .map_err(Into::into)
                        .map(|_| Status::Starting),
                }
            }
        }
    }

    pub fn as_named_process(&self) -> NamedProcess {
        NamedProcess::new(self.alias(), self.process_id() as usize)
    }

    pub fn p2p_listen_addr(&self) -> SocketAddr {
        self.p2p_listen_address
    }

    pub fn p2p_public_address(&self) -> Multiaddr {
        self.p2p_public_address.clone()
    }

    pub fn rest_address(&self) -> SocketAddr {
        self.rest_socket_addr
    }

    pub fn alias(&self) -> NodeAlias {
        self.alias.to_string()
    }

    pub fn rest(&self) -> JormungandrRest {
        JormungandrRest::new(self.rest_uri())
    }

    pub fn rest_debug(&self) -> JormungandrRest {
        let mut rest = JormungandrRest::new(self.rest_uri());
        rest.enable_logger();
        rest
    }

    pub fn secure_rest<P: AsRef<Path>>(&self, cert: P) -> JormungandrRest {
        JormungandrRest::new_with_cert(self.rest_uri(), cert)
    }

    pub fn shutdown(&self) {
        let jcli: JCli = Default::default();
        jcli.rest().v0().shutdown(self.rest_uri());
    }

    pub fn address(&self) -> SocketAddr {
        jormungandr_lib::multiaddr::to_tcp_socket_addr(&self.p2p_public_address).unwrap()
    }

    pub fn correct_state_verifier(&self) -> JormungandrStateVerifier {
        JormungandrStateVerifier::new(self.rest())
    }

    pub fn log_stats(&self) {
        println!("{:?}", self.rest().stats());
    }

    pub fn assert_no_errors_in_log_with_message(&self, message: &str) {
        self.logger.assert_no_errors(message);
    }

    pub fn assert_no_errors_in_log(&self) {
        self.logger.assert_no_errors("");
    }

    pub fn check_no_errors_in_log(&self) -> Result<(), JormungandrError> {
        let error_lines = self
            .logger
            .get_lines_with_level(LogLevel::ERROR)
            .collect::<Vec<_>>();

        if !error_lines.is_empty() {
            return Err(JormungandrError::ErrorInLogs {
                logs: self.logger.get_log_content(),
                error_lines: format!("{:?}", error_lines),
            });
        }
        Ok(())
    }

    pub fn rest_uri(&self) -> String {
        uri_from_socket_addr(self.rest_socket_addr)
    }

    pub fn fees(&self) -> LinearFee {
        self.block0_configuration()
            .blockchain_configuration
            .linear_fees
    }

    pub fn genesis_block_hash(&self) -> Hash {
        self.block0_configuration.to_block().header().id().into()
    }

    pub fn block0_configuration(&self) -> &Block0Configuration {
        &self.block0_configuration
    }

    pub fn pid(&self) -> u32 {
        self.child.id()
    }

    pub fn explorer(&self) -> Explorer {
        Explorer::new(self.rest_socket_addr.to_string())
    }

    pub fn to_trusted_peer(&self) -> TrustedPeer {
        TrustedPeer {
            address: self.p2p_public_address.clone(),
            id: None,
        }
    }

    pub fn time_era(&self) -> TimeEra {
        let block_date = self.explorer().current_time();

        TimeEra::new(
            (block_date.slot() as u64).into(),
            chain_time::Epoch(block_date.epoch()),
            self.block0_configuration
                .blockchain_configuration
                .slots_per_epoch
                .into(),
        )
    }

    pub fn to_remote(&self) -> RemoteJormungandr {
        let mut builder = RemoteJormungandrBuilder::new(self.alias.clone());
        builder.with_rest(self.rest_socket_addr);
        builder.build()
    }

    pub fn steal_temp_dir(&mut self) -> Option<TestingDirectory> {
        self.temp_dir.take()
    }

    pub fn stop(mut self) {
        match self.child.kill() {
            Err(e) => println!("Could not kill {}: {}", self.alias, e),
            Ok(_) => {
                println!("Successfully killed {}", self.alias);
            }
        }
    }
}

impl Drop for JormungandrProcess {
    fn drop(&mut self) {
        // There's no kill like overkill
        let _ = self.child.kill();
        // FIXME: These should be better done in a test harness
        self.child.wait().unwrap();

        utils::persist_dir_on_panic(
            self.temp_dir.take(),
            vec![("node.log", &self.log_content())],
        );
    }
}

impl SyncNode for JormungandrProcess {
    fn alias(&self) -> NodeAlias {
        self.alias()
    }

    fn last_block_height(&self) -> u32 {
        let docs = self.rest().stats().unwrap();
        docs.stats
            .expect("no stats object in response")
            .last_block_height
            .expect("last_block_height field is missing")
            .parse()
            .unwrap()
    }

    fn log_stats(&self) {
        println!("{:?}", self.rest().stats());
    }

    fn tip(&self) -> Hash {
        self.rest().tip().expect("cannot get tip from rest")
    }

    fn log_content(&self) -> String {
        self.logger.get_log_content()
    }

    fn get_lines_with_error_and_invalid(&self) -> Vec<String> {
        self.logger
            .get_lines_with_level(LogLevel::ERROR)
            .map(|x| x.to_string())
            .collect()
    }

    fn is_running(&self) -> bool {
        matches!(
            self.status(&StartupVerificationMode::Log),
            Ok(Status::Running)
        )
    }
}

#[derive(Debug, Error)]
pub enum ShutdownError {
    #[error("node wasn't properly shutdown after {timeout} s. Log file: {log_content}")]
    Timeout { timeout: u64, log_content: String },
    #[error("error(s) while starting")]
    Jormungandr(#[from] JormungandrError),
    #[error("process still active")]
    ProcessStillActive(#[from] std::io::Error),
}