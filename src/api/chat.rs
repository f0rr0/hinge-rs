use crate::client::HingeClient;
use crate::errors::HingeError;
use crate::models::{
    ExportChatInput, ExportChatResult, SendMessagePayload, SendbirdChannelHandle,
    SendbirdChannelsResponse, SendbirdCloseRequest, SendbirdGetMessagesInput, SendbirdGroupChannel,
    SendbirdMessage, SendbirdMessagesResponse, SendbirdReadResponse,
};
use crate::storage::Storage;
use crate::ws::SendbirdWsSubscription;

pub struct ChatApi<'a, S: Storage + Clone> {
    pub(super) client: &'a mut HingeClient<S>,
}

impl<S: Storage + Clone> ChatApi<'_, S> {
    pub async fn init_flow(&mut self) -> Result<serde_json::Value, HingeError> {
        self.client.sendbird_init_flow().await
    }

    pub async fn credentials(&mut self) -> Result<serde_json::Value, HingeError> {
        self.client.sendbird_creds().await
    }

    pub async fn channels_raw(
        &mut self,
        user_id: &str,
        limit: usize,
    ) -> Result<serde_json::Value, HingeError> {
        self.client
            .sendbird_list_my_group_channels(user_id, limit)
            .await
    }

    pub async fn channels(&mut self, limit: usize) -> Result<SendbirdChannelsResponse, HingeError> {
        self.client.sendbird_list_channels_typed(limit).await
    }

    pub async fn channel_raw(
        &mut self,
        channel_url: &str,
    ) -> Result<serde_json::Value, HingeError> {
        self.client.sendbird_get_channel(channel_url).await
    }

    pub async fn channel(&mut self, channel_url: &str) -> Result<SendbirdGroupChannel, HingeError> {
        self.client.sendbird_get_channel_typed(channel_url).await
    }

    pub async fn messages(
        &mut self,
        input: SendbirdGetMessagesInput,
    ) -> Result<SendbirdMessagesResponse, HingeError> {
        let message_ts = input.message_ts.parse::<i64>().unwrap_or(0);
        self.client
            .sendbird_get_messages(&input.channel_url, message_ts, input.prev_limit as usize)
            .await
    }

    pub async fn full_messages(
        &mut self,
        channel_url: &str,
    ) -> Result<Vec<SendbirdMessage>, HingeError> {
        self.client.sendbird_get_full_messages(channel_url).await
    }

    pub async fn export_chat(
        &mut self,
        input: ExportChatInput,
    ) -> Result<ExportChatResult, HingeError> {
        self.client.export_chat(input).await
    }

    pub async fn create_distinct_dm(
        &mut self,
        self_user_id: &str,
        peer_user_id: &str,
        data_mm: i32,
    ) -> Result<serde_json::Value, HingeError> {
        self.client
            .sendbird_create_distinct_dm(self_user_id, peer_user_id, data_mm)
            .await
    }

    pub async fn get_or_create_dm_channel(
        &mut self,
        self_user_id: &str,
        peer_user_id: &str,
    ) -> Result<String, HingeError> {
        self.client
            .sendbird_get_or_create_dm_channel(self_user_id, peer_user_id)
            .await
    }

    pub async fn ensure_dm_with(
        &mut self,
        partner_id: &str,
    ) -> Result<SendbirdChannelHandle, HingeError> {
        self.client.ensure_sendbird_channel_with(partner_id).await
    }

    pub async fn send_message(
        &self,
        payload: SendMessagePayload,
    ) -> Result<serde_json::Value, HingeError> {
        self.client.send_message(payload).await
    }

    pub async fn subscribe(
        &mut self,
    ) -> Result<
        (
            tokio::sync::mpsc::UnboundedSender<String>,
            tokio::sync::broadcast::Receiver<String>,
        ),
        HingeError,
    > {
        self.client.sendbird_ws_subscribe().await
    }

    pub async fn subscribe_events(&mut self) -> Result<SendbirdWsSubscription, HingeError> {
        let (commands, raw) = self.client.sendbird_ws_subscribe().await?;
        Ok(SendbirdWsSubscription::new(commands, raw))
    }

    pub async fn send_ws_command(&mut self, command: String) -> Result<(), HingeError> {
        self.client.sendbird_ws_send_command(command).await
    }

    pub async fn mark_read(
        &mut self,
        channel_url: &str,
    ) -> Result<SendbirdReadResponse, HingeError> {
        self.client
            .sendbird_ws_send_read_and_wait(channel_url)
            .await
    }

    pub async fn mark_read_fire_and_forget(&mut self, channel_url: &str) -> Result<(), HingeError> {
        self.client.sendbird_ws_send_read(channel_url).await
    }

    pub async fn ping(&mut self) -> Result<(), HingeError> {
        self.client.sendbird_ws_send_ping().await
    }

    pub async fn typing_start(&mut self, channel_url: &str) -> Result<(), HingeError> {
        self.client.sendbird_ws_send_typing_start(channel_url).await
    }

    pub async fn typing_end(&mut self, channel_url: &str) -> Result<(), HingeError> {
        self.client.sendbird_ws_send_typing_end(channel_url).await
    }

    pub async fn enter_channel(&mut self, channel_url: &str) -> Result<(), HingeError> {
        self.client
            .sendbird_ws_send_enter_channel(channel_url)
            .await
    }

    pub async fn exit_channel(&mut self, channel_url: &str) -> Result<(), HingeError> {
        self.client.sendbird_ws_send_exit_channel(channel_url).await
    }

    pub async fn ack_message(
        &mut self,
        channel_url: &str,
        message_id: &str,
    ) -> Result<(), HingeError> {
        self.client
            .sendbird_ws_send_message_ack(channel_url, message_id)
            .await
    }

    pub async fn close_ws(&mut self, request: SendbirdCloseRequest) -> Result<(), HingeError> {
        self.client
            .sendbird_ws_close(request.code, request.reason)
            .await
    }

    pub async fn ensure_ws_connected(&mut self) -> Result<bool, HingeError> {
        self.client.sendbird_ws_ensure_connected().await
    }
}
