// 提取器管道的集成测试
// 验证 pipeline.run() 能正确将所有 drain 数据发送到 sink 链

use crate::extractor_pipeline::ExtractorPipeline;
use crate::test::extractor_pipeline::{
    forward_sink::ForwardSink, mock_creater::MockCreater, mock_reader::MockReader,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[tokio::test]
// 验证：ExtractorPipeline 在 run 后，drain 的数据全部到达 ForwardSink
// 数据含 1 行标题 + 3 行数据，其中 2 行 ready，1 行需 drain → 共 3 条数据到达 sink
async fn run_should_send_all_drain_data_to_sink_chain() {
    // 模拟 4 行数据：1 行标题 + 3 行正文（第二列标记是否 ready）
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
    let sink = ForwardSink {
        sink_count: Arc::clone(&first_sink_count),
    };

    // 创建并运行管道
    let pipeline =
        ExtractorPipeline::new(reader, creater, Box::new(sink), 0).expect("pipeline new");
    pipeline.run().await.expect("pipeline run");

    // 应收到 3 条 drain 数据（2 条 ready + 1 条 drain）
    assert_eq!(first_sink_count.load(Ordering::Relaxed), 3);
}
