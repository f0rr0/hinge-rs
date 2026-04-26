use crate::client::HingeClient;
use crate::errors::HingeError;
use crate::models::{
    ConnectionDetailApi, ConnectionsResponse, MatchNoteResponse, StandoutsResponse,
};
use crate::storage::Storage;

pub struct ConnectionsApi<'a, S: Storage + Clone> {
    pub(super) client: &'a mut HingeClient<S>,
}

impl<S: Storage + Clone> ConnectionsApi<'_, S> {
    pub async fn list(&self) -> Result<ConnectionsResponse, HingeError> {
        self.client.get_connections_v2().await
    }

    pub async fn detail(&self, subject_id: &str) -> Result<ConnectionDetailApi, HingeError> {
        self.client.get_connection_detail(subject_id).await
    }

    pub async fn match_note(&self, subject_id: &str) -> Result<MatchNoteResponse, HingeError> {
        self.client.get_connection_match_note(subject_id).await
    }

    pub async fn standouts(&self) -> Result<StandoutsResponse, HingeError> {
        self.client.get_standouts().await
    }
}
