use super::HingeClient;
use crate::errors::HingeError;
use crate::models::{
    ConnectionDetailApi, ConnectionsResponse, MatchNoteResponse, StandoutsResponse,
};
use crate::storage::Storage;

impl<S: Storage + Clone> HingeClient<S> {
    pub async fn get_connections_v2(&self) -> Result<ConnectionsResponse, HingeError> {
        let url = format!("{}/connection/v2", self.settings.base_url);
        let res = self.http_get(&url).await?;
        self.parse_response(res).await
    }

    pub async fn get_connection_detail(
        &self,
        subject_id: &str,
    ) -> Result<ConnectionDetailApi, HingeError> {
        let url = format!(
            "{}/connection/subject/{}",
            self.settings.base_url, subject_id
        );
        let res = self.http_get(&url).await?;
        self.parse_response(res).await
    }

    pub async fn get_connection_match_note(
        &self,
        subject_id: &str,
    ) -> Result<MatchNoteResponse, HingeError> {
        let url = format!(
            "{}/connection/v2/matchnote/{}",
            self.settings.base_url, subject_id
        );
        let res = self.http_get(&url).await?;
        self.parse_response(res).await
    }

    pub async fn get_standouts(&self) -> Result<StandoutsResponse, HingeError> {
        let url = format!("{}/standouts/v3", self.settings.base_url);
        let res = self.http_get(&url).await?;
        self.parse_response(res).await
    }
}
