use crate::common::jcli::command::rest::v0::MessageCommand;
use assert_cmd::assert::OutputAssertExt;
use assert_fs::{fixture::FileWriteStr, NamedTempFile};
use chain_impl_mockchain::fragment::FragmentId;
use jormungandr_lib::interfaces::{FragmentLog, FragmentsProcessingSummary};
use jortestkit::prelude::ProcessOutput;

pub struct Message {
    message_command: MessageCommand,
}

impl Message {
    pub fn new(message_command: MessageCommand) -> Self {
        Self { message_command }
    }

    pub fn post<S: Into<String>>(self, fragment: &str, host: S) -> FragmentId {
        let transaction_file = NamedTempFile::new("transaction.hash").unwrap();
        transaction_file.write_str(fragment).unwrap();

        let response = self
            .message_command
            .post(transaction_file.path(), host.into())
            .build()
            .assert()
            .success()
            .get_output()
            .as_single_line();

        let summary: FragmentsProcessingSummary =
            serde_json::from_str(&response).expect("not a valid json");
        summary.accepted.get(0).cloned().unwrap_or_else(|| {
            panic!(
                "the fragment was rejected by the node: {:?}",
                summary.rejected
            )
        })
    }

    pub fn logs<S: Into<String>>(self, host: S) -> Vec<FragmentLog> {
        let content = self
            .message_command
            .logs(host.into())
            .build()
            .assert()
            .success()
            .get_output()
            .as_lossy_string();

        serde_yaml::from_str(&content).expect("Failed to parse fragment log")
    }
}
