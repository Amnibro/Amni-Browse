use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct IceCandidate {
    pub candidate: String,
    pub sdp_mid: Option<String>,
    pub sdp_m_line_index: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SdpType { Offer, Answer, Pranswer, Rollback }

#[derive(Debug, Clone)]
pub struct SessionDescription {
    pub sdp_type: SdpType,
    pub sdp: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RtcSignalingState {
    Stable, HaveLocalOffer, HaveRemoteOffer,
    HaveLocalPranswer, HaveRemotePranswer, Closed,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RtcIceConnectionState {
    New, Checking, Connected, Completed, Disconnected, Failed, Closed,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataChannelState { Connecting, Open, Closing, Closed }

#[derive(Debug, Clone)]
pub struct RtcDataChannel {
    pub id: String,
    pub label: String,
    pub state: DataChannelState,
    pub buffered_messages: Vec<String>,
}

impl RtcDataChannel {
    pub fn send(&mut self, msg: &str) {
        if self.state == DataChannelState::Open {
            self.buffered_messages.push(msg.to_string());
        }
    }

    pub fn close(&mut self) {
        self.state = DataChannelState::Closed;
    }

    pub fn drain_messages(&mut self) -> Vec<String> {
        std::mem::take(&mut self.buffered_messages)
    }
}

pub struct RtcPeerConnection {
    pub id: String,
    pub signaling_state: RtcSignalingState,
    pub ice_connection_state: RtcIceConnectionState,
    pub local_description: Option<SessionDescription>,
    pub remote_description: Option<SessionDescription>,
    pub ice_candidates: Vec<IceCandidate>,
    pub data_channels: HashMap<String, RtcDataChannel>,
    pub local_streams: Vec<String>,
    pub remote_streams: Vec<String>,
}

impl RtcPeerConnection {
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            signaling_state: RtcSignalingState::Stable,
            ice_connection_state: RtcIceConnectionState::New,
            local_description: None,
            remote_description: None,
            ice_candidates: Vec::new(),
            data_channels: HashMap::new(),
            local_streams: Vec::new(),
            remote_streams: Vec::new(),
        }
    }

    pub fn create_offer(&self) -> SessionDescription {
        SessionDescription {
            sdp_type: SdpType::Offer,
            sdp: format!(
                "v=0\r\no=- {} 2 IN IP4 127.0.0.1\r\ns=-\r\nt=0 0\r\n\
                 a=group:BUNDLE 0\r\na=msid-semantic: WMS\r\n",
                chrono::Utc::now().timestamp()
            ),
        }
    }

    pub fn create_answer(&self) -> SessionDescription {
        SessionDescription {
            sdp_type: SdpType::Answer,
            sdp: format!(
                "v=0\r\no=- {} 2 IN IP4 127.0.0.1\r\ns=-\r\nt=0 0\r\n\
                 a=group:BUNDLE 0\r\na=msid-semantic: WMS\r\n",
                chrono::Utc::now().timestamp()
            ),
        }
    }

    pub fn set_local_description(&mut self, desc: SessionDescription) {
        self.signaling_state = match desc.sdp_type {
            SdpType::Offer => RtcSignalingState::HaveLocalOffer,
            SdpType::Answer => RtcSignalingState::Stable,
            SdpType::Pranswer => RtcSignalingState::HaveLocalPranswer,
            SdpType::Rollback => RtcSignalingState::Stable,
        };
        self.local_description = Some(desc);
    }

    pub fn set_remote_description(&mut self, desc: SessionDescription) {
        self.signaling_state = match desc.sdp_type {
            SdpType::Offer => RtcSignalingState::HaveRemoteOffer,
            SdpType::Answer => RtcSignalingState::Stable,
            SdpType::Pranswer => RtcSignalingState::HaveRemotePranswer,
            SdpType::Rollback => RtcSignalingState::Stable,
        };
        self.remote_description = Some(desc);
    }

    pub fn add_ice_candidate(&mut self, candidate: IceCandidate) {
        self.ice_candidates.push(candidate);
    }

    pub fn create_data_channel(&mut self, label: &str) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        self.data_channels.insert(id.clone(), RtcDataChannel {
            id: id.clone(), label: label.to_string(),
            state: DataChannelState::Connecting, buffered_messages: Vec::new(),
        });
        id
    }

    pub fn add_stream(&mut self, stream_id: &str) {
        if !self.local_streams.contains(&stream_id.to_string()) {
            self.local_streams.push(stream_id.to_string());
        }
    }

    pub fn close(&mut self) {
        self.signaling_state = RtcSignalingState::Closed;
        self.ice_connection_state = RtcIceConnectionState::Closed;
        for ch in self.data_channels.values_mut() { ch.close(); }
    }
}

pub struct RtcManager {
    pub connections: HashMap<String, RtcPeerConnection>,
}

impl RtcManager {
    pub fn new() -> Self { Self { connections: HashMap::new() } }

    pub fn create_connection(&mut self) -> String {
        let pc = RtcPeerConnection::new();
        let id = pc.id.clone();
        self.connections.insert(id.clone(), pc);
        id
    }

    pub fn get_connection(&self, id: &str) -> Option<&RtcPeerConnection> {
        self.connections.get(id)
    }

    pub fn get_connection_mut(&mut self, id: &str) -> Option<&mut RtcPeerConnection> {
        self.connections.get_mut(id)
    }

    pub fn close_connection(&mut self, id: &str) {
        if let Some(pc) = self.connections.get_mut(id) { pc.close(); }
    }

    pub fn connection_count(&self) -> usize { self.connections.len() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_connection_offer_answer() {
        let mut pc = RtcPeerConnection::new();
        assert_eq!(pc.signaling_state, RtcSignalingState::Stable);
        let offer = pc.create_offer();
        assert_eq!(offer.sdp_type, SdpType::Offer);
        assert!(offer.sdp.contains("v=0"));
        pc.set_local_description(offer);
        assert_eq!(pc.signaling_state, RtcSignalingState::HaveLocalOffer);
    }

    #[test]
    fn test_ice_candidates() {
        let mut pc = RtcPeerConnection::new();
        pc.add_ice_candidate(IceCandidate {
            candidate: "candidate:1 1 UDP 2130706431 192.168.1.1 5000 typ host".into(),
            sdp_mid: Some("0".into()), sdp_m_line_index: Some(0),
        });
        assert_eq!(pc.ice_candidates.len(), 1);
        assert!(pc.ice_candidates[0].candidate.contains("host"));
    }

    #[test]
    fn test_data_channel() {
        let mut pc = RtcPeerConnection::new();
        let ch_id = pc.create_data_channel("chat");
        let ch = pc.data_channels.get_mut(&ch_id).unwrap();
        assert_eq!(ch.label, "chat");
        assert_eq!(ch.state, DataChannelState::Connecting);
        ch.state = DataChannelState::Open;
        ch.send("hello");
        ch.send("world");
        let msgs = ch.drain_messages();
        assert_eq!(msgs, vec!["hello", "world"]);
        assert!(ch.buffered_messages.is_empty());
    }

    #[test]
    fn test_rtc_manager() {
        let mut mgr = RtcManager::new();
        assert_eq!(mgr.connection_count(), 0);
        let id = mgr.create_connection();
        assert_eq!(mgr.connection_count(), 1);
        assert!(mgr.get_connection(&id).is_some());
        mgr.close_connection(&id);
        let pc = mgr.get_connection(&id).unwrap();
        assert_eq!(pc.signaling_state, RtcSignalingState::Closed);
        assert_eq!(pc.ice_connection_state, RtcIceConnectionState::Closed);
    }

    #[test]
    fn test_add_stream_and_close() {
        let mut pc = RtcPeerConnection::new();
        pc.add_stream("stream-1");
        pc.add_stream("stream-1");
        assert_eq!(pc.local_streams.len(), 1);
        pc.add_stream("stream-2");
        assert_eq!(pc.local_streams.len(), 2);
        pc.create_data_channel("dc1");
        pc.close();
        assert_eq!(pc.signaling_state, RtcSignalingState::Closed);
        assert!(pc.data_channels.values().all(|ch| ch.state == DataChannelState::Closed));
    }
}
