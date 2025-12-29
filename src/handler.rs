use nerve_protocol::codec::encode;
use nerve_protocol::frame::OwnedFrame;
use nerve_protocol::types::{FrameFlags, MessageType, RequestId};

use crate::state::RequestState;

pub fn handle_search(
    frame: OwnedFrame,
    state: &mut RequestState,
)->Option<Vec<u8>>{
    let request_id = RequestId(frame.header.request_id);

    if state.is_cancelled(request_id){
        return None;
    }

    // v0.1 stub result (bytes)
    let result = b"stub search result";

    let reply = encode(
        MessageType::SearchResult,
        FrameFlags::FINAL,
        request_id,
        result,
    ).ok()?;
    
    Some(reply)
}