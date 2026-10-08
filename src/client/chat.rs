use super::HingeClient;
use crate::errors::HingeError;
use crate::models::SendMessagePayload;
use crate::storage::Storage;
use crate::ws::is_guest_refusal;
use std::time::Duration;
use uuid::Uuid;

/// How long to wait for Sendbird to echo a sent message back before giving up on it.
const SEND_MESSAGE_ACK_TIMEOUT: Duration = Duration::from_secs(15);

impl<S: Storage + Clone> HingeClient<S> {
    /// Send a chat message to a match.
    ///
    /// Messages travel over the Sendbird WebSocket, which is what the Hinge app does and the
    /// only path that works: Hinge's own `POST /message/send` answers 400 for every body shape,
    /// so it is not attempted. The return value is Sendbird's `MESG` echo — the stored message,
    /// including its `msg_id` and server timestamp.
    ///
    /// `payload.origin` and `payload.match_message` are accepted for API continuity but are not
    /// part of a WS send; only `subject_id` (to resolve the channel) and `message_data.message`
    /// reach the wire. `dedup_id` is filled in when absent so callers keep a stable id to log.
    pub async fn send_message(
        &mut self,
        mut payload: SendMessagePayload,
    ) -> Result<serde_json::Value, HingeError> {
        let self_user_id = self
            .hinge_auth
            .as_ref()
            .ok_or_else(|| HingeError::Auth("hinge token missing".into()))?
            .identity_id
            .clone();

        // Failing to resolve the channel used to be swallowed and the send attempted anyway,
        // which turned a stale Sendbird session into an unrelated-looking 400 from Hinge. There
        // is nowhere to send without a channel, so the real error is the one worth returning.
        let channel_url = self
            .sendbird_get_or_create_dm_channel(&self_user_id, &payload.subject_id)
            .await?;

        if payload.dedup_id.is_none() {
            payload.dedup_id = Some(Uuid::new_v4().to_string().to_uppercase());
        }

        let text = payload.message_data.message.clone();
        let sent = self
            .sendbird_ws_send_message_and_wait(&channel_url, &text, SEND_MESSAGE_ACK_TIMEOUT)
            .await;

        // "Guest is not allowed" means the WS came up unauthenticated, which reads cannot
        // reveal and the expiry check cannot predict. Re-authenticate from scratch and send
        // once more; any other refusal is final and is returned as-is.
        match sent {
            Err(HingeError::SendbirdRefused { code, message })
                if is_guest_refusal(code, &message) =>
            {
                log::warn!("[sendbird] send refused as guest - re-authenticating and retrying");
                self.invalidate_sendbird_session();
                self.sendbird_ws_close(None, None).await.ok();
                self.sendbird_ws_send_message_and_wait(
                    &channel_url,
                    &text,
                    SEND_MESSAGE_ACK_TIMEOUT,
                )
                .await
            }
            other => other,
        }
    }
}
