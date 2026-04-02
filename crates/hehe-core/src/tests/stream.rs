use crate::stream::{StopReason, StreamAggregator, StreamChunk};
use crate::types::MessageId;
use tracing::info;

#[test]
fn test_aggregator_text() {
    super::init_tracing();

    let mut agg = StreamAggregator::new();
    agg.push(StreamChunk::MessageStart {
        message_id: MessageId::new(),
    });
    info!("pushed MessageStart");

    agg.push(StreamChunk::TextDelta {
        text: "Hello ".into(),
    });
    agg.push(StreamChunk::TextDelta {
        text: "world!".into(),
    });
    info!(text = agg.text(), "aggregated text deltas");

    agg.push(StreamChunk::MessageEnd {
        stop_reason: Some(StopReason::EndTurn),
    });
    info!(complete = agg.is_complete(), "stream finished");

    assert_eq!(agg.text(), "Hello world!");
    assert!(agg.is_complete());
    assert_eq!(agg.stop_reason(), Some(&StopReason::EndTurn));
    assert!(!agg.has_tool_use());
}

#[test]
fn test_aggregator_tool_use() {
    super::init_tracing();

    let mut agg = StreamAggregator::new();
    agg.push(StreamChunk::ToolUseStart {
        id: "c1".into(),
        name: "read_file".into(),
    });
    agg.push(StreamChunk::ToolUseDelta {
        id: "c1".into(),
        input_delta: r#"{"path"#.into(),
    });
    agg.push(StreamChunk::ToolUseDelta {
        id: "c1".into(),
        input_delta: r#":"/tmp"}"#.into(),
    });
    agg.push(StreamChunk::MessageEnd {
        stop_reason: Some(StopReason::ToolUse),
    });

    info!(tool_count = agg.tool_use_count(), "tool uses aggregated");

    assert!(agg.has_tool_use());
    assert_eq!(agg.tool_use_count(), 1);
}

#[test]
fn test_aggregator_clear() {
    super::init_tracing();

    let mut agg = StreamAggregator::new();
    agg.push(StreamChunk::TextDelta {
        text: "data".into(),
    });
    info!(before = agg.text(), "before clear");
    assert_eq!(agg.text(), "data");

    agg.clear();
    info!(after = agg.text(), "after clear");
    assert_eq!(agg.text(), "");
}
