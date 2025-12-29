use crawler::SearchEngine;
use crawler::search::filters::SortBy;
use nerve_protocol::codec::encode;
use nerve_protocol::frame::OwnedFrame;
use nerve_protocol::types::{FrameFlags, MessageType, RequestId};

use crate::state::RequestState;

pub fn handle_search(
    frame: OwnedFrame,
    state: &mut RequestState,
    engine: &SearchEngine,
)->Option<Vec<u8>>{
    let request_id = RequestId(frame.header.request_id);

    if state.is_cancelled(request_id){
        return None;
    }

    // v0.1 defaults
    let query = std::str::from_utf8(&frame.payload).ok()?;
    
    let result = engine.search(
        query,
        10,
        0,
        crawler::search::filters::SearchFilter::new(),
        SortBy::Relevance,
        true,
        false,
    ).ok()?;

    // serialize results
    let payload = serde_json::to_vec(&result).ok()?;
    Some(encode(MessageType::SearchResult, FrameFlags::FINAL, request_id, &payload).ok()?)
}