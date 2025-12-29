use std::io::Cursor;

use crawler::search::SearchSchema;
use nerve_protocol::constants::{MAGIC, VERSION};
use nerve_protocol::frame::{FrameHeader, OwnedFrame};
use nerve_protocol::io::FrameReader;
use nerve_protocol::types::{FrameFlags, MessageType, RequestId};
use tempfile::tempdir;
use tantivy::{doc, Index};

use nerve_search_adapter::handler::handle_search;
use nerve_search_adapter::state::RequestState;

fn build_search_engine_with_sample() -> SearchEngineTestHarness {
    let dir = tempdir().expect("tempdir");
    let schema = SearchSchema::build();
    let index = Index::create_in_dir(dir.path(), schema.schema.clone()).expect("index create");

    let mut writer = index.writer(50_000_000).expect("writer");
    writer
        .add_document(doc!(
            schema.url_field => "https://example.com/rust",
            schema.title_field => "Rust search adapter",
            schema.content_field => "rust search adapter integration",
            schema.domain_field => "example.com",
            schema.quality_field => "0.9",
            schema.pagerank_field => 0.42f64,
            schema.tfidf_field => 0.21f64
        ))
        .expect("add doc");
    writer.commit().expect("commit");

    let engine = crawler::SearchEngine::new(dir.path()).expect("search engine");
    SearchEngineTestHarness { _dir: dir, engine }
}

struct SearchEngineTestHarness {
    _dir: tempfile::TempDir,
    engine: crawler::SearchEngine,
}

#[test]
fn handle_search_returns_search_result_frame() {
    let harness = build_search_engine_with_sample();
    let mut state = RequestState::new();

    let payload = b"rust".to_vec();
    let header = FrameHeader {
        magic: MAGIC,
        version: VERSION,
        msg_type: MessageType::SearchQuery as u8,
        flags: FrameFlags::empty().bits(),
        request_id: 42,
        payload_length: payload.len() as u32,
    };
    let frame = OwnedFrame { header, payload };

    let bytes = handle_search(frame, &mut state, &harness.engine)
        .expect("expected search reply bytes");

    let mut reader = FrameReader::new();
    let mut cursor = Cursor::new(bytes);
    let frames = reader.read_from(&mut cursor).expect("decode frame");
    assert_eq!(frames.len(), 1);
    let reply = &frames[0];

    assert_eq!(reply.header.msg_type, MessageType::SearchResult as u8);
    assert_eq!(reply.header.request_id, 42);
    let reply_flags = FrameFlags::from_bits_truncate(reply.header.flags);
    assert!(reply_flags.contains(FrameFlags::FINAL));
    assert!(!reply.payload.is_empty());

    let json: serde_json::Value = serde_json::from_slice(&reply.payload).expect("json payload");
    assert!(json.is_array(), "payload should be JSON array");
    assert!(!json.as_array().unwrap().is_empty(), "results should not be empty");
}

#[test]
fn handle_search_is_suppressed_when_cancelled() {
    let harness = build_search_engine_with_sample();
    let mut state = RequestState::new();

    let request_id = RequestId(99);
    state.cancel(request_id);

    let payload = b"rust".to_vec();
    let header = FrameHeader {
        magic: MAGIC,
        version: VERSION,
        msg_type: MessageType::SearchQuery as u8,
        flags: FrameFlags::empty().bits(),
        request_id: request_id.0,
        payload_length: payload.len() as u32,
    };
    let frame = OwnedFrame { header, payload };

    let bytes = handle_search(frame, &mut state, &harness.engine);
    assert!(bytes.is_none(), "cancelled request must not emit output");
}
