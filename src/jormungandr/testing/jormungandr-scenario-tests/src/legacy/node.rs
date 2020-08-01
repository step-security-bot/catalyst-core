#![allow(dead_code)]

/// Specialized node which is supposed to be compatible with 5 last jormungandr releases
use crate::{
    legacy::LegacySettings,
    node::{Error, ProgressBarController, Result, Status},
    style, Context,
};
use chain_impl_mockchain::{
    block::Block,
    fragment::{Fragment, FragmentId},
    header::HeaderId,
};
use jormungandr_integration_tests::{
    common::jormungandr::logger::JormungandrLogger, mock::client::JormungandrClient,
};
use jormungandr_lib::{
    crypto::hash::Hash,
    interfaces::{
        EnclaveLeaderId, FragmentLog, FragmentStatus, Info, Log, LogEntry, LogOutput, PeerRecord,
        PeerStats,
    },
};
pub use jormungandr_testing_utils::testing::{
    network_builder::{
        LeadershipMode, NodeAlias, NodeBlock0, NodeSetting, PersistenceMode, Settings,
    },
    FragmentNode, FragmentNodeError, MemPoolCheck,
};

use futures::executor::block_on;
use indicatif::ProgressBar;
use rand_core::RngCore;
use yaml_rust::{Yaml, YamlLoader};

use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// send query to a running node
#[derive(Clone)]
pub struct LegacyNodeController {
    alias: NodeAlias,
    grpc_client: JormungandrClient,
    settings: LegacySettings,
    progress_bar: ProgressBarController,
    status: Arc<Mutex<Status>>,
}

pub struct LegacyNode {
    alias: NodeAlias,

    #[allow(unused)]
    dir: PathBuf,

    process: Child,

    progress_bar: ProgressBarController,
    node_settings: LegacySettings,
    status: Arc<Mutex<Status>>,
}

const NODE_CONFIG: &str = "node_config.yaml";
const NODE_SECRET: &str = "node_secret.yaml";
const NODE_STORAGE: &str = "storage.db";
const NODE_LOG: &str = "node.log";

impl LegacyNodeController {
    pub fn alias(&self) -> &NodeAlias {
        &self.alias
    }

    pub fn status(&self) -> Status {
        *self.status.lock().unwrap()
    }

    pub fn check_running(&self) -> bool {
        self.status() == Status::Running
    }

    fn path(&self, path: &str) -> String {
        format!("{}/{}", self.base_url(), path)
    }

    pub fn address(&self) -> poldercast::Address {
        self.settings.config.p2p.public_address.clone()
    }

    pub fn progress_bar(&self) -> &ProgressBarController {
        &self.progress_bar
    }

    fn post(&self, path: &str, body: Vec<u8>) -> Result<reqwest::blocking::Response> {
        self.progress_bar.log_info(format!("POST '{}'", path));

        let client = reqwest::blocking::Client::new();
        let res = client
            .post(&format!("{}/{}", self.base_url(), path))
            .body(body)
            .send();

        match res {
            Err(err) => {
                self.progress_bar
                    .log_err(format!("Failed to send request {}", &err));
                Err(err.into())
            }
            Ok(r) => Ok(r),
        }
    }

    fn get(&self, path: &str) -> Result<reqwest::blocking::Response> {
        self.progress_bar.log_info(format!("GET '{}'", path));

        match reqwest::blocking::get(&format!("{}/{}", self.base_url(), path)) {
            Err(err) => {
                self.progress_bar
                    .log_err(format!("Failed to send request {}", &err));
                Err(err.into())
            }
            Ok(r) => Ok(r),
        }
    }

    fn base_url(&self) -> String {
        format!("http://{}/api/v0", self.settings.config.rest.listen.clone())
    }

    pub fn send_fragment(&self, fragment: Fragment) -> Result<MemPoolCheck> {
        use chain_core::property::Fragment as _;
        use chain_core::property::Serialize as _;

        let raw = fragment.serialize_as_vec().unwrap();
        let fragment_id = fragment.id();

        let response = self.post("message", raw.clone())?;
        self.progress_bar
            .log_info(format!("Fragment '{}' sent", fragment_id,));

        let res = response.error_for_status_ref();
        if let Err(err) = res {
            self.progress_bar.log_err(format!(
                "Fragment '{}' ({}) fail to send: {}",
                hex::encode(&raw),
                fragment_id,
                err,
            ));
        }

        Ok(MemPoolCheck::new(fragment_id))
    }

    pub fn log(&self, info: &str) {
        self.progress_bar.log_info(info);
    }

    pub fn tip(&self) -> Result<Hash> {
        let hash = self.get("tip")?.text()?;

        let hash = hash.parse().map_err(Error::InvalidHeaderId)?;

        self.progress_bar.log_info(format!("tip '{}'", hash));

        Ok(hash)
    }

    pub fn blocks_to_tip(&self, from: HeaderId) -> Result<Vec<Block>> {
        block_on(self.grpc_client.pull_blocks_to_tip(from)).map_err(Error::InvalidGrpcCall)
    }

    pub fn network_stats(&self) -> Result<Vec<PeerStats>> {
        let response_text = self.get("network/stats")?.text()?;
        self.progress_bar
            .log_info(format!("network/stats: {}", response_text));

        let network_stats: Vec<PeerStats> = if response_text.is_empty() {
            Vec::new()
        } else {
            serde_json::from_str(&response_text).map_err(Error::InvalidNetworkStats)?
        };
        Ok(network_stats)
    }

    pub fn p2p_quarantined(&self) -> Result<Vec<PeerRecord>> {
        let response_text = self.get("network/p2p/quarantined")?.text()?;

        self.progress_bar
            .log_info(format!("network/p2p_quarantined: {}", response_text));

        let network_stats: Vec<PeerRecord> = if response_text.is_empty() {
            Vec::new()
        } else {
            serde_json::from_str(&response_text).map_err(Error::InvalidNetworkStats)?
        };
        Ok(network_stats)
    }

    pub fn p2p_non_public(&self) -> Result<Vec<PeerRecord>> {
        let response_text = self.get("network/p2p/non_public")?.text()?;

        self.progress_bar
            .log_info(format!("network/non_publicS: {}", response_text));

        let network_stats: Vec<PeerRecord> = if response_text.is_empty() {
            Vec::new()
        } else {
            serde_json::from_str(&response_text).map_err(Error::InvalidNetworkStats)?
        };
        Ok(network_stats)
    }

    pub fn p2p_available(&self) -> Result<Vec<PeerRecord>> {
        let response_text = self.get("network/p2p/available")?.text()?;

        self.progress_bar
            .log_info(format!("network/available: {}", response_text));

        let network_stats: Vec<PeerRecord> = if response_text.is_empty() {
            Vec::new()
        } else {
            serde_json::from_str(&response_text).map_err(Error::InvalidNetworkStats)?
        };
        Ok(network_stats)
    }

    pub fn p2p_view(&self) -> Result<Vec<Info>> {
        let response_text = self.get("network/p2p/view")?.text()?;

        self.progress_bar
            .log_info(format!("network/view: {}", response_text));

        let network_stats: Vec<Info> = if response_text.is_empty() {
            Vec::new()
        } else {
            serde_json::from_str(&response_text).map_err(Error::InvalidNetworkStats)?
        };
        Ok(network_stats)
    }

    pub fn all_blocks_hashes(&self) -> Result<Vec<HeaderId>> {
        let genesis_hash = self
            .genesis_block_hash()
            .expect("Cannot download genesis hash");
        self.blocks_hashes_to_tip(genesis_hash)
    }

    pub fn blocks_hashes_to_tip(&self, from: HeaderId) -> Result<Vec<HeaderId>> {
        Ok(self
            .blocks_to_tip(from)
            .unwrap()
            .iter()
            .map(|x| x.header.hash())
            .collect())
    }

    pub fn genesis_block_hash(&self) -> Result<HeaderId> {
        Ok(block_on(self.grpc_client.get_genesis_block_hash()))
    }

    pub fn block(&self, header_hash: &HeaderId) -> Result<Block> {
        use chain_core::mempack::{ReadBuf, Readable as _};

        let mut resp = self.get(&format!("block/{}", header_hash))?;
        let mut bytes = Vec::new();
        resp.copy_to(&mut bytes)?;
        let block = Block::read(&mut ReadBuf::from(&bytes)).map_err(Error::InvalidBlock)?;

        self.progress_bar.log_info(format!(
            "block{} ({}) '{}'",
            block.header.chain_length(),
            block.header.block_date(),
            header_hash,
        ));

        Ok(block)
    }

    pub fn fragment_logs(&self) -> Result<HashMap<FragmentId, FragmentLog>> {
        let logs = self.get("fragment/logs")?.text()?;

        let logs: Vec<FragmentLog> = if logs.is_empty() {
            Vec::new()
        } else {
            serde_json::from_str(&logs).map_err(Error::InvalidFragmentLogs)?
        };

        self.progress_bar
            .log_info(format!("fragment logs ({})", logs.len()));

        let logs = logs
            .into_iter()
            .map(|log| (log.fragment_id().clone().into_hash(), log))
            .collect();

        Ok(logs)
    }

    pub fn wait_fragment(&self, duration: Duration, check: MemPoolCheck) -> Result<FragmentStatus> {
        let max_try = 50;
        for _ in 0..max_try {
            let logs = self.fragment_logs()?;

            if let Some(log) = logs.get(&check.fragment_id()) {
                use jormungandr_lib::interfaces::FragmentStatus::*;
                let status = log.status().clone();
                match log.status() {
                    Pending => {
                        self.progress_bar.log_info(format!(
                            "Fragment '{}' is still pending",
                            check.fragment_id()
                        ));
                    }
                    Rejected { reason } => {
                        self.progress_bar.log_info(format!(
                            "Fragment '{}' rejected: {}",
                            check.fragment_id(),
                            reason
                        ));
                        return Ok(status);
                    }
                    InABlock { date, block } => {
                        self.progress_bar.log_info(format!(
                            "Fragment '{}' in block: {} ({})",
                            check.fragment_id(),
                            block,
                            date
                        ));
                        return Ok(status);
                    }
                }
            } else {
                return Err(Error::FragmentNotInMemPoolLogs {
                    alias: self.alias().to_string(),
                    fragment_id: *check.fragment_id(),
                    logs: self.logger().get_lines_from_log().collect(),
                });
            }
            std::thread::sleep(duration);
        }

        Err(Error::FragmentIsPendingForTooLong {
            fragment_id: *check.fragment_id(),
            duration: Duration::from_secs(duration.as_secs() * max_try),
            alias: self.alias().to_string(),
            logs: self.logger().get_lines_from_log().collect(),
        })
    }

    pub fn leaders(&self) -> Result<Vec<EnclaveLeaderId>> {
        let leaders = self.get("leaders")?.text()?;
        let leaders: Vec<EnclaveLeaderId> = if leaders.is_empty() {
            Vec::new()
        } else {
            serde_json::from_str(&leaders).map_err(Error::InvalidEnclaveLeaderIds)?
        };

        self.progress_bar
            .log_info(format!("leaders ids ({})", leaders.len()));

        Ok(leaders)
    }

    pub fn promote(&self) -> Result<EnclaveLeaderId> {
        let path = "leaders";
        let secrets = self.settings.secrets();
        self.progress_bar.log_info(format!("POST '{}'", &path));
        let response = reqwest::blocking::Client::new()
            .post(&self.path(path))
            .json(&secrets)
            .send()?;

        self.progress_bar
            .log_info(format!("Leader promotion for '{}' sent", self.alias()));

        let res = response.error_for_status_ref();
        if let Err(err) = res {
            self.progress_bar.log_err(format!(
                "Leader promotion for '{}' fail to sent: {}",
                self.alias(),
                err,
            ));
        }

        let leader_id: EnclaveLeaderId = response.json()?;
        Ok(leader_id)
    }

    pub fn demote(&self, leader_id: u32) -> Result<()> {
        let path = format!("leaders/{}", leader_id);
        self.progress_bar.log_info(format!("DELETE '{}'", &path));
        let response = reqwest::blocking::Client::new()
            .delete(&self.path(&path))
            .send()?;

        self.progress_bar
            .log_info(format!("Leader demote for '{}' sent", self.alias()));

        let res = response.error_for_status_ref();
        if let Err(err) = res {
            self.progress_bar.log_err(format!(
                "Leader demote for '{}' fail to sent: {}",
                self.alias(),
                err,
            ));
        }
        Ok(())
    }

    pub fn stats(&self) -> Result<Yaml> {
        let stats = self.get("node/stats")?.text()?;
        let docs = YamlLoader::load_from_str(&stats)?;
        Ok(docs.get(0).unwrap().clone())
    }

    pub fn log_stats(&self) {
        self.progress_bar
            .log_info(format!("node stats ({:?})", self.stats()));
    }

    pub fn wait_for_bootstrap(&self) -> Result<()> {
        let max_try = 20;
        let sleep = Duration::from_secs(8);
        for _ in 0..max_try {
            let stats = self.stats();
            match stats {
                Ok(stats) => {
                    if stats["state"].as_str().unwrap() == "Running" {
                        self.log_stats();
                        return Ok(());
                    }
                }
                Err(err) => self
                    .progress_bar
                    .log_info(format!("node stats failure({:?})", err)),
            };
            std::thread::sleep(sleep);
        }
        Err(Error::NodeFailedToBootstrap {
            alias: self.alias().to_string(),
            duration: Duration::from_secs(sleep.as_secs() * max_try),
            logs: self.logger().get_lines_from_log().collect(),
        })
    }

    pub fn wait_for_shutdown(&self) -> Result<()> {
        let max_try = 2;
        let sleep = Duration::from_secs(2);
        for _ in 0..max_try {
            if self.stats().is_err() && self.ports_are_opened() {
                return Ok(());
            };
            std::thread::sleep(sleep);
        }
        Err(Error::NodeFailedToShutdown {
            alias: self.alias().to_string(),
            message: format!(
                "node is still up after {} s from sending shutdown request",
                sleep.as_secs()
            ),
            logs: self.logger().get_lines_from_log().collect(),
        })
    }

    fn ports_are_opened(&self) -> bool {
        self.port_opened(self.settings.config.rest.listen.port())
            && self.port_opened(
                self.settings
                    .config
                    .p2p
                    .public_address
                    .to_socketaddr()
                    .unwrap()
                    .port(),
            )
    }

    fn port_opened(&self, port: u16) -> bool {
        use std::net::TcpListener;
        TcpListener::bind(("127.0.0.1", port)).is_ok()
    }

    pub fn is_up(&self) -> bool {
        let stats = self.stats();
        match stats {
            Ok(stats) => stats["state"].as_str().unwrap() == "Running",
            Err(_) => false,
        }
    }

    pub fn shutdown(&self) -> Result<()> {
        let result = self.get("shutdown")?.text()?;

        if result == "" {
            self.progress_bar.log_info("shuting down");
            self.wait_for_shutdown()
        } else {
            Err(Error::NodeFailedToShutdown {
                alias: self.alias().to_string(),
                message: result,
                logs: self.logger().get_lines_from_log().collect(),
            })
        }
    }

    pub fn logger(&self) -> JormungandrLogger {
        let log_file = self
            .settings
            .config
            .log
            .as_ref()
            .unwrap()
            .file_path()
            .unwrap();
        JormungandrLogger::new(log_file)
    }
}

impl LegacyNode {
    pub fn alias(&self) -> &NodeAlias {
        &self.alias
    }

    pub fn controller(&self) -> LegacyNodeController {
        let p2p_address = format!("{}", self.node_settings.config().p2p.public_address);

        LegacyNodeController {
            alias: self.alias().clone(),
            grpc_client: JormungandrClient::from_address(&p2p_address)
                .expect("cannot setup grpc client"),
            settings: self.node_settings.clone(),
            status: self.status.clone(),
            progress_bar: self.progress_bar.clone(),
        }
    }

    pub fn spawn<R: RngCore>(
        jormungandr: &Path,
        context: &Context<R>,
        progress_bar: ProgressBar,
        alias: &str,
        node_settings: &mut LegacySettings,
        block0: NodeBlock0,
        working_dir: &Path,
        peristence_mode: PersistenceMode,
    ) -> Result<Self> {
        let mut command = Command::new(jormungandr);
        let dir = working_dir.join(alias);
        std::fs::DirBuilder::new().recursive(true).create(&dir)?;

        let progress_bar = ProgressBarController::new(
            progress_bar,
            format!("{}@{}", alias, node_settings.config().rest.listen),
            context.progress_bar_mode(),
        );

        let config_file = dir.join(NODE_CONFIG);
        let config_secret = dir.join(NODE_SECRET);
        let log_file = dir.join(NODE_LOG);

        let format = "plain";
        let level = context.log_level();
        node_settings.config.log = Some(Log(vec![
            LogEntry {
                format: format.to_string(),
                level: level.to_string(),
                output: LogOutput::Stderr,
            },
            LogEntry {
                format: format.to_string(),
                level: level.to_string(),
                output: LogOutput::File(log_file),
            },
        ]));

        if peristence_mode == PersistenceMode::Persistent {
            let path_to_storage = dir.join(NODE_STORAGE);
            node_settings.config.storage = Some(path_to_storage);
        }

        serde_yaml::to_writer(
            std::fs::File::create(&config_file).map_err(|e| Error::CannotCreateFile {
                path: config_file.clone(),
                cause: e,
            })?,
            node_settings.config(),
        )
        .map_err(|e| Error::CannotWriteYamlFile {
            path: config_file.clone(),
            cause: e,
        })?;

        serde_yaml::to_writer(
            std::fs::File::create(&config_secret).map_err(|e| Error::CannotCreateFile {
                path: config_secret.clone(),
                cause: e,
            })?,
            node_settings.secrets(),
        )
        .map_err(|e| Error::CannotWriteYamlFile {
            path: config_secret.clone(),
            cause: e,
        })?;

        command.arg("--config");
        command.arg(&config_file);

        match block0 {
            NodeBlock0::File(path) => {
                command.arg("--genesis-block");
                command.arg(&path);
                command.arg("--secret");
                command.arg(&config_secret);
            }
            NodeBlock0::Hash(hash) => {
                command.args(&["--genesis-block-hash", &hash.to_string()]);
            }
        }

        command.stderr(Stdio::piped());

        let process = command.spawn().map_err(Error::CannotSpawnNode)?;

        let node = LegacyNode {
            alias: alias.into(),

            dir,

            process,

            progress_bar,
            node_settings: node_settings.clone(),
            status: Arc::new(Mutex::new(Status::Running)),
        };

        node.progress_bar_start();

        Ok(node)
    }

    pub fn capture_logs(&mut self) {
        let stderr = self.process.stderr.take().unwrap();
        let reader = BufReader::new(stderr);
        for line_result in reader.lines() {
            let line = line_result.expect("failed to read a line from log output");
            self.progress_bar.log_info(&line);
        }
    }

    pub fn wait(&mut self) {
        match self.process.wait() {
            Err(err) => {
                self.progress_bar.log_err(&err);
                self.progress_bar_failure();
                self.set_status(Status::Failure);
            }
            Ok(status) => {
                if status.success() {
                    self.progress_bar_success();
                } else {
                    self.progress_bar.log_err(&status);
                    self.progress_bar_failure()
                }
                self.set_status(Status::Exit(status));
            }
        }
    }

    fn progress_bar_start(&self) {
        self.progress_bar.set_style(
            indicatif::ProgressStyle::default_spinner()
                .template("{spinner:.green} {wide_msg}")
                .tick_chars(style::TICKER),
        );
        self.progress_bar.enable_steady_tick(100);
        self.progress_bar.set_message(&format!(
            "{} {} ... [{}]",
            *style::icons::jormungandr,
            style::binary.apply_to(self.alias()),
            self.node_settings.config().rest.listen,
        ));
    }

    fn progress_bar_failure(&self) {
        self.progress_bar.finish_with_message(&format!(
            "{} {} {}",
            *style::icons::jormungandr,
            style::binary.apply_to(self.alias()),
            style::error.apply_to(*style::icons::failure)
        ));
    }

    fn progress_bar_success(&self) {
        self.progress_bar.finish_with_message(&format!(
            "{} {} {}",
            *style::icons::jormungandr,
            style::binary.apply_to(self.alias()),
            style::success.apply_to(*style::icons::success)
        ));
    }

    fn set_status(&self, status: Status) {
        *self.status.lock().unwrap() = status
    }
}