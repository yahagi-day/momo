//! WebRTC signaling message types.

use serde::{Deserialize, Serialize};

/// Messages sent from client to server via WebSocket.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    Answer { sdp: String },
    IceCandidate {
        candidate: String,
        sdp_mid: Option<String>,
        sdp_m_line_index: Option<u16>,
    },
    SubscribeTrack { stream_id: String },
    UnsubscribeTrack { stream_id: String },
}

/// Messages sent from server to client via WebSocket.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    Offer { sdp: String },
    IceCandidate {
        candidate: String,
        sdp_mid: Option<String>,
        sdp_m_line_index: Option<u16>,
    },
    TrackAdded { stream_id: String, mid: String },
    TrackRemoved { stream_id: String },
    Error { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_answer_roundtrip() {
        let msg = ClientMessage::Answer { sdp: "v=0\r\n".into() };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ClientMessage = serde_json::from_str(&json).unwrap();
        match parsed {
            ClientMessage::Answer { sdp } => assert_eq!(sdp, "v=0\r\n"),
            _ => panic!("expected Answer"),
        }
    }

    #[test]
    fn client_subscribe_roundtrip() {
        let msg = ClientMessage::SubscribeTrack { stream_id: "input".into() };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ClientMessage = serde_json::from_str(&json).unwrap();
        match parsed {
            ClientMessage::SubscribeTrack { stream_id } => assert_eq!(stream_id, "input"),
            _ => panic!("expected SubscribeTrack"),
        }
    }

    #[test]
    fn server_offer_roundtrip() {
        let msg = ServerMessage::Offer { sdp: "v=0\r\n".into() };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ServerMessage = serde_json::from_str(&json).unwrap();
        match parsed {
            ServerMessage::Offer { sdp } => assert_eq!(sdp, "v=0\r\n"),
            _ => panic!("expected Offer"),
        }
    }

    #[test]
    fn server_ice_candidate_roundtrip() {
        let msg = ServerMessage::IceCandidate {
            candidate: "candidate:1 1 UDP 2130706431 192.168.1.1 5000 typ host".into(),
            sdp_mid: Some("0".into()),
            sdp_m_line_index: Some(0),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ServerMessage = serde_json::from_str(&json).unwrap();
        match parsed {
            ServerMessage::IceCandidate { candidate, sdp_mid, sdp_m_line_index } => {
                assert!(candidate.contains("candidate:1"));
                assert_eq!(sdp_mid, Some("0".into()));
                assert_eq!(sdp_m_line_index, Some(0));
            }
            _ => panic!("expected IceCandidate"),
        }
    }

    #[test]
    fn server_error_roundtrip() {
        let msg = ServerMessage::Error { message: "not available".into() };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ServerMessage = serde_json::from_str(&json).unwrap();
        match parsed {
            ServerMessage::Error { message } => assert_eq!(message, "not available"),
            _ => panic!("expected Error"),
        }
    }

    #[test]
    fn client_ice_candidate_roundtrip() {
        let msg = ClientMessage::IceCandidate {
            candidate: "candidate:1".into(),
            sdp_mid: None,
            sdp_m_line_index: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ClientMessage = serde_json::from_str(&json).unwrap();
        match parsed {
            ClientMessage::IceCandidate { candidate, sdp_mid, sdp_m_line_index } => {
                assert_eq!(candidate, "candidate:1");
                assert!(sdp_mid.is_none());
                assert!(sdp_m_line_index.is_none());
            }
            _ => panic!("expected IceCandidate"),
        }
    }
}
