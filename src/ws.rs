use crate::errors::HingeError;
use crate::models::SendbirdSyevEvent;
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, mpsc};

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum SendbirdWsEvent {
    SessionKey {
        key: String,
    },
    Read {
        #[serde(default)]
        req_id: Option<String>,
        payload: serde_json::Value,
    },
    Typing {
        event: SendbirdSyevEvent,
    },
    Ping {
        payload: serde_json::Value,
    },
    Pong {
        payload: serde_json::Value,
    },
    Close {
        code: Option<u16>,
        reason: Option<String>,
    },
    Raw {
        frame: String,
    },
}

pub struct SendbirdWsSubscription {
    commands: mpsc::UnboundedSender<String>,
    raw: broadcast::Receiver<String>,
}

impl SendbirdWsSubscription {
    pub fn new(commands: mpsc::UnboundedSender<String>, raw: broadcast::Receiver<String>) -> Self {
        Self { commands, raw }
    }

    pub fn commands(&self) -> mpsc::UnboundedSender<String> {
        self.commands.clone()
    }

    pub fn resubscribe_raw(&self) -> broadcast::Receiver<String> {
        self.raw.resubscribe()
    }

    pub async fn recv(&mut self) -> Result<SendbirdWsEvent, HingeError> {
        let frame = self.recv_raw().await?;
        parse_sendbird_ws_frame(&frame)
    }

    pub async fn recv_raw(&mut self) -> Result<String, HingeError> {
        self.raw
            .recv()
            .await
            .map_err(|err| HingeError::Http(format!("sendbird ws receive failed: {err}")))
    }
}

pub fn parse_sendbird_ws_frame(frame: &str) -> Result<SendbirdWsEvent, HingeError> {
    if let Some(rest) = frame.strip_prefix("__SESSION_KEY__:") {
        return Ok(SendbirdWsEvent::SessionKey {
            key: rest.to_string(),
        });
    }

    if let Some(rest) = frame.strip_prefix("__SYEV__:") {
        let event = serde_json::from_str::<SendbirdSyevEvent>(rest)
            .map_err(|err| HingeError::Serde(err.to_string()))?;
        return Ok(SendbirdWsEvent::Typing { event });
    }

    if let Some(rest) = frame.strip_prefix("__CLOSE__:") {
        let mut parts = rest.splitn(2, ':');
        let code = parts.next().and_then(|part| part.parse::<u16>().ok());
        let reason = parts
            .next()
            .filter(|part| !part.is_empty())
            .map(ToOwned::to_owned);
        return Ok(SendbirdWsEvent::Close { code, reason });
    }

    parse_prefixed_json(frame)
}

fn parse_prefixed_json(frame: &str) -> Result<SendbirdWsEvent, HingeError> {
    let Some(start) = frame.find('{') else {
        return Ok(SendbirdWsEvent::Raw {
            frame: frame.to_string(),
        });
    };
    let (prefix, json) = frame.split_at(start);
    let payload = serde_json::from_str::<serde_json::Value>(json)
        .map_err(|err| HingeError::Serde(err.to_string()))?;

    match prefix {
        "LOGI" => Ok(SendbirdWsEvent::SessionKey {
            key: payload
                .get("key")
                .and_then(|value| value.as_str())
                .unwrap_or_default()
                .to_string(),
        }),
        "READ" => Ok(SendbirdWsEvent::Read {
            req_id: payload
                .get("req_id")
                .and_then(|value| value.as_str())
                .map(ToOwned::to_owned),
            payload,
        }),
        "SYEV" => {
            let event = serde_json::from_value::<SendbirdSyevEvent>(payload)
                .map_err(|err| HingeError::Serde(err.to_string()))?;
            Ok(SendbirdWsEvent::Typing { event })
        }
        "PING" => Ok(SendbirdWsEvent::Ping { payload }),
        "PONG" => Ok(SendbirdWsEvent::Pong { payload }),
        _ => Ok(SendbirdWsEvent::Raw {
            frame: frame.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_logi_session_key() {
        let event = parse_sendbird_ws_frame(r#"LOGI{"key":"session-key","user_id":"u1"}"#)
            .expect("LOGI frame should parse");
        assert_eq!(
            event,
            SendbirdWsEvent::SessionKey {
                key: "session-key".to_string()
            }
        );
    }

    #[test]
    fn parses_read_ack() {
        let event = parse_sendbird_ws_frame(
            r#"READ{"req_id":"r1","channel_id":1,"channel_url":"c","channel_type":"group"}"#,
        )
        .expect("READ frame should parse");
        match event {
            SendbirdWsEvent::Read { req_id, payload } => {
                assert_eq!(req_id.as_deref(), Some("r1"));
                assert_eq!(payload["channel_url"], "c");
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[test]
    fn parses_typing_event() {
        let event = parse_sendbird_ws_frame(
            r#"SYEV{"cat":10900,"channel_url":"c","channel_type":"group","ts":1,"sts":1}"#,
        )
        .expect("SYEV frame should parse");
        match event {
            SendbirdWsEvent::Typing { event } => {
                assert_eq!(event.cat, SendbirdSyevEvent::CATEGORY_TYPING_START);
                assert_eq!(event.channel_url, "c");
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[test]
    fn parses_ping_pong_and_close() {
        let ping =
            parse_sendbird_ws_frame(r#"PING{"req_id":"p1"}"#).expect("PING frame should parse");
        assert!(matches!(ping, SendbirdWsEvent::Ping { .. }));

        let pong =
            parse_sendbird_ws_frame(r#"PONG{"req_id":"p1"}"#).expect("PONG frame should parse");
        assert!(matches!(pong, SendbirdWsEvent::Pong { .. }));

        let close =
            parse_sendbird_ws_frame("__CLOSE__:1000:done").expect("close sentinel should parse");
        assert_eq!(
            close,
            SendbirdWsEvent::Close {
                code: Some(1000),
                reason: Some("done".to_string())
            }
        );
    }

    #[test]
    fn keeps_unknown_frames_raw() {
        let event = parse_sendbird_ws_frame("NOPE{}").expect("unknown frame should parse");
        assert_eq!(
            event,
            SendbirdWsEvent::Raw {
                frame: "NOPE{}".to_string()
            }
        );
    }
}
