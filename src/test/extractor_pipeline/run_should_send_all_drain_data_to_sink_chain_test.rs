use crate::extractor_pipeline::ExtractorPipeline;
use crate::test::extractor_pipeline::{
    forward_sink::ForwardSink, leaf_sink::LeafSink, mock_creater::MockCreater,
    mock_reader::MockReader,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn run_should_send_all_drain_data_to_sink_chain() {
    let reader = MockReader {
        rows: vec![
            vec!["header_a".to_string(), "header_b".to_string()],
            vec!["row_1".to_string(), "1".to_string()],
            vec!["row_2".to_string(), "0".to_string()],
            vec!["row_3".to_string(), "1".to_string()],
        ],
    };

    let creater = MockCreater::new();
    let first_sink_count = Arc::new(AtomicUsize::new(0));
    let next_sink_count = Arc::new(AtomicUsize::new(0));
    let sink = ForwardSink {
        sink_count: Arc::clone(&first_sink_count),
        next_sink: Box::new(LeafSink {
            sink_count: Arc::clone(&next_sink_count),
        }),
    };

    let pipeline =
        ExtractorPipeline::new(reader, creater, Box::new(sink), 0).expect("pipeline new");
    pipeline.run().expect("pipeline run");

    assert_eq!(first_sink_count.load(Ordering::Relaxed), 3);
    assert_eq!(next_sink_count.load(Ordering::Relaxed), 3);
}
