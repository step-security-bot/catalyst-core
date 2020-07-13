use super::vote_options;
use crate::db::models::vote_options::VoteOptions;
use crate::db::{views_schema::full_proposals_info, DB};
use diesel::Queryable;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Category {
    #[serde(alias = "categoryId")]
    pub category_id: String,
    #[serde(alias = "categoryName")]
    pub category_name: String,
    #[serde(alias = "categoryDescription")]
    pub category_description: String,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Proposer {
    #[serde(alias = "proposerName")]
    pub proposer_name: String,
    #[serde(alias = "proposerEmail")]
    pub proposer_email: String,
    #[serde(alias = "proposerUrl")]
    pub proposer_url: String,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Proposal {
    #[serde(alias = "internalId")]
    pub internal_id: i32,
    #[serde(alias = "proposalId")]
    pub proposal_id: String,
    #[serde(alias = "category")]
    pub proposal_category: Category,
    #[serde(alias = "proposalTitle")]
    pub proposal_title: String,
    #[serde(alias = "proposalSummary")]
    pub proposal_summary: String,
    #[serde(alias = "proposalProblem")]
    pub proposal_problem: String,
    #[serde(alias = "proposalSolution")]
    pub proposal_solution: String,
    #[serde(alias = "proposalPublicKey")]
    pub proposal_public_key: String,
    #[serde(alias = "proposalFunds")]
    pub proposal_funds: i64,
    #[serde(alias = "proposalUrl")]
    pub proposal_url: String,
    #[serde(alias = "proposalFilesUrl")]
    pub proposal_files_url: String,
    pub proposer: Proposer,
    #[serde(alias = "chainProposalId")]
    #[serde(serialize_with = "crate::utils::serde::serialize_bin_as_str")]
    #[serde(deserialize_with = "crate::utils::serde::deserialize_string_as_bytes")]
    pub chain_proposal_id: Vec<u8>,
    #[serde(alias = "chainProposalIndex")]
    pub chain_proposal_index: i64,
    #[serde(alias = "chainVoteOptions")]
    pub chain_vote_options: VoteOptions,
    #[serde(alias = "chainVoteplanId")]
    pub chain_voteplan_id: String,
    #[serde(alias = "chainVoteStartTime")]
    #[serde(serialize_with = "crate::utils::serde::serialize_unix_timestamp_as_rfc3339")]
    #[serde(deserialize_with = "crate::utils::serde::deserialize_unix_timestamp_from_rfc3339")]
    pub chain_vote_start_time: i64,
    #[serde(alias = "chainVoteEndTime")]
    #[serde(serialize_with = "crate::utils::serde::serialize_unix_timestamp_as_rfc3339")]
    #[serde(deserialize_with = "crate::utils::serde::deserialize_unix_timestamp_from_rfc3339")]
    pub chain_vote_end_time: i64,
    #[serde(alias = "chainCommitteeEndTime")]
    #[serde(serialize_with = "crate::utils::serde::serialize_unix_timestamp_as_rfc3339")]
    #[serde(deserialize_with = "crate::utils::serde::deserialize_unix_timestamp_from_rfc3339")]
    pub chain_committee_end_time: i64,
    #[serde(alias = "chainVoteplanPayload")]
    pub chain_voteplan_payload: String,
    #[serde(alias = "fundId")]
    pub fund_id: i32,
}

impl Queryable<full_proposals_info::SqlType, DB> for Proposal {
    // The row is the row, for now it cannot be any other type, may change when the DB schema changes
    #[allow(clippy::type_complexity)]
    type Row = (
        // 0 ->id
        i32,
        // 1 -> proposal_id
        String,
        // 2-> category_name
        String,
        // 3 -> proposal_title
        String,
        // 4 -> proposal_summary
        String,
        // 5 -> proposal_problem
        String,
        // 6 -> proposal_solution
        String,
        // 7 -> proposal_public_key
        String,
        // 8 -> proposal_funds
        i64,
        // 9 -> proposal_url
        String,
        // 10 -> proposal_files_url,
        String,
        // 11 -> proposer_name
        String,
        // 12 -> proposer_contact
        String,
        // 13 -> proposer_url
        String,
        // 14 -> chain_proposal_id
        Vec<u8>,
        // 15 -> chain_proposal_index
        i64,
        // 16 -> chain_vote_options
        String,
        // 17 -> chain_voteplan_id
        String,
        // 18 -> chain_vote_starttime
        i64,
        // 29 -> chain_vote_endtime
        i64,
        // 20 -> chain_committee_end_time
        i64,
        // 21 -> chain_voteplan_payload
        String,
        // 22 -> fund_id
        i32,
    );

    fn build(row: Self::Row) -> Self {
        Proposal {
            internal_id: row.0,
            proposal_id: row.1,
            proposal_category: Category {
                category_id: "".to_string(),
                category_name: row.2,
                category_description: "".to_string(),
            },
            proposal_title: row.3,
            proposal_summary: row.4,
            proposal_problem: row.5,
            proposal_solution: row.6,
            proposal_public_key: row.7,
            proposal_funds: row.8,
            proposal_url: row.9,
            proposal_files_url: row.10,
            proposer: Proposer {
                proposer_name: row.11,
                proposer_email: row.12,
                proposer_url: row.13,
            },
            chain_proposal_id: row.14,
            chain_proposal_index: row.15,
            chain_vote_options: vote_options::VoteOptions::parse_coma_separated_value(&row.16),
            chain_voteplan_id: row.17,
            chain_vote_start_time: row.18,
            chain_vote_end_time: row.19,
            chain_committee_end_time: row.20,
            chain_voteplan_payload: row.21,
            fund_id: row.22,
        }
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::db::{
        models::vote_options::VoteOptions,
        schema::{proposals, voteplans},
        DBConnectionPool,
    };
    use chrono::Utc;
    use diesel::{ExpressionMethods, RunQueryDsl};

    pub fn get_test_proposal() -> Proposal {
        Proposal {
            internal_id: 1,
            proposal_id: "1".to_string(),
            proposal_category: Category {
                category_id: "".to_string(),
                category_name: "foo_category_name".to_string(),
                category_description: "".to_string(),
            },
            proposal_title: "the proposal".to_string(),
            proposal_summary: "the proposal summary".to_string(),
            proposal_problem: "the proposal problem".to_string(),
            proposal_solution: "the proposal solution".to_string(),
            proposal_public_key: "pubkey".to_string(),
            proposal_funds: 10000,
            proposal_url: "http://foo.bar".to_string(),
            proposal_files_url: "http://foo.bar/files".to_string(),
            proposer: Proposer {
                proposer_name: "tester".to_string(),
                proposer_email: "tester@tester.tester".to_string(),
                proposer_url: "http://tester.tester".to_string(),
            },
            chain_proposal_id: b"foobar".to_vec(),
            chain_proposal_index: 0,
            chain_vote_options: VoteOptions::parse_coma_separated_value("b,a,r"),
            chain_voteplan_id: "voteplain_id".to_string(),
            chain_vote_start_time: Utc::now().timestamp(),
            chain_vote_end_time: Utc::now().timestamp(),
            chain_committee_end_time: Utc::now().timestamp(),
            chain_voteplan_payload: "none".to_string(),
            fund_id: 1,
        }
    }

    pub fn populate_db_with_proposal(proposal: &Proposal, pool: &DBConnectionPool) {
        let connection = pool.get().unwrap();

        // insert the proposal information
        let values = (
            proposals::proposal_id.eq(proposal.proposal_id.clone()),
            proposals::proposal_category.eq(proposal.proposal_category.category_name.clone()),
            proposals::proposal_title.eq(proposal.proposal_title.clone()),
            proposals::proposal_summary.eq(proposal.proposal_summary.clone()),
            proposals::proposal_problem.eq(proposal.proposal_problem.clone()),
            proposals::proposal_solution.eq(proposal.proposal_solution.clone()),
            proposals::proposal_public_key.eq(proposal.proposal_public_key.clone()),
            proposals::proposal_funds.eq(proposal.proposal_funds.clone()),
            proposals::proposal_url.eq(proposal.proposal_url.clone()),
            proposals::proposal_files_url.eq(proposal.proposal_files_url.clone()),
            proposals::proposer_name.eq(proposal.proposer.proposer_name.clone()),
            proposals::proposer_contact.eq(proposal.proposer.proposer_email.clone()),
            proposals::proposer_url.eq(proposal.proposer.proposer_url.clone()),
            proposals::chain_proposal_id.eq(proposal.chain_proposal_id.clone()),
            proposals::chain_proposal_index.eq(proposal.chain_proposal_index),
            proposals::chain_vote_options.eq(proposal.chain_vote_options.as_csv_string()),
            proposals::chain_voteplan_id.eq(proposal.chain_voteplan_id.clone()),
        );
        diesel::insert_into(proposals::table)
            .values(values)
            .execute(&connection)
            .unwrap();

        // insert the related fund voteplan information
        let voteplan_values = (
            voteplans::chain_voteplan_id.eq(proposal.chain_voteplan_id.clone()),
            voteplans::chain_vote_start_time.eq(proposal.chain_vote_start_time),
            voteplans::chain_vote_end_time.eq(proposal.chain_vote_end_time),
            voteplans::chain_committee_end_time.eq(proposal.chain_committee_end_time),
            voteplans::chain_voteplan_payload.eq(proposal.chain_voteplan_payload.clone()),
            voteplans::fund_id.eq(proposal.fund_id),
        );

        diesel::insert_into(voteplans::table)
            .values(voteplan_values)
            .execute(&connection)
            .unwrap();
    }
}