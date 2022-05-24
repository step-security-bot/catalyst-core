use crate::community_advisors::models::AdvisorReviewRow;
use crate::utils;
use color_eyre::Report;
use vit_servicing_station_lib::db::models::community_advisors_reviews::AdvisorReview;

use std::path::Path;
impl AdvisorReviewRow {
    fn as_advisor_review(&self) -> AdvisorReview {
        AdvisorReview {
            id: 0,
            proposal_id: self.proposal_id.parse().unwrap(),
            assessor: self.assessor.clone(),
            impact_alignment_rating_given: self.impact_alignment_rating as i32,
            impact_alignment_note: self.impact_alignment_note.clone(),
            feasibility_rating_given: self.feasibility_rating as i32,
            feasibility_note: self.feasibility_note.clone(),
            auditability_rating_given: self.auditability_rating as i32,
            auditability_note: self.auditability_note.clone(),
            ranking: self.score().into(),
        }
    }
}

pub fn read_vca_reviews_aggregated_file(filepath: &Path) -> Result<Vec<AdvisorReview>, Report> {
    Ok(
        utils::csv::load_data_from_csv::<AdvisorReviewRow, b','>(filepath)?
            .into_iter()
            .map(|review| review.as_advisor_review())
            .collect(),
    )
}
