use std::sync::Arc;
use crate::data::repo::artifact_repo::ArtifactRepo;
use crate::data::repo::category_repo::CategoryRepo;
use crate::data::repo::object_repo::ObjectRepo;
use crate::data::repo::publishing_repo::PublishingRepo;
use crate::data::repo::report_repo::ReportRepo;
use crate::data::repo::review_repo::ReviewRepo;
use crate::data::repo::search_repo::SearchRepo;
use crate::data::repo::validation_repo::ValidationRepo;
use crate::net::etag_handler::EtagHandler;

#[derive(Clone)]
pub struct ClientState {
    pub object_repo: Arc<ObjectRepo>,
    pub category_repo: Arc<CategoryRepo>,
    pub search_repo: Arc<SearchRepo>,
    pub review_repo: Arc<ReviewRepo>,
    pub publishing_repo: Arc<PublishingRepo>,
    pub validation_repo: Arc<ValidationRepo>,
    pub artifact_repo: Arc<ArtifactRepo>,
    pub report_repo: Arc<ReportRepo>,
    pub etag_handler: Arc<EtagHandler>,
}
