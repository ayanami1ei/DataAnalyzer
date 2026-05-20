# Details

Date : 2026-05-20 09:34:21

Directory /home/ayanami/桌面/软件设计/DataAnalyzer/src

Total : 88 files,  4567 codes, 8 comments, 778 blanks, all 5353 lines

[Summary](results.md) / Details / [Diff Summary](diff.md) / [Diff Details](diff-details.md)

## Files
| filename | language | code | comment | blank | total |
| :--- | :--- | ---: | ---: | ---: | ---: |
| [src/checker/data\_check\_rule.rs](/src/checker/data_check_rule.rs) | Rust | 4 | 0 | 2 | 6 |
| [src/checker/geometry\_check\_rule.rs](/src/checker/geometry_check_rule.rs) | Rust | 40 | 1 | 8 | 49 |
| [src/checker/mod.rs](/src/checker/mod.rs) | Rust | 2 | 0 | 1 | 3 |
| [src/data/data.rs](/src/data/data.rs) | Rust | 1 | 0 | 1 | 2 |
| [src/data/data\_impl/flow.rs](/src/data/data_impl/flow.rs) | Rust | 16 | 0 | 6 | 22 |
| [src/data/data\_impl/mod.rs](/src/data/data_impl/mod.rs) | Rust | 98 | 0 | 21 | 119 |
| [src/data/data\_impl/schema.rs](/src/data/data_impl/schema.rs) | Rust | 91 | 0 | 17 | 108 |
| [src/data/data\_impl/value.rs](/src/data/data_impl/value.rs) | Rust | 56 | 0 | 10 | 66 |
| [src/data/data\_pair.rs](/src/data/data_pair.rs) | Rust | 9 | 0 | 3 | 12 |
| [src/data/mod.rs](/src/data/mod.rs) | Rust | 4 | 0 | 0 | 4 |
| [src/data/object\_io/config\_reader.rs](/src/data/object_io/config_reader.rs) | Rust | 22 | 0 | 7 | 29 |
| [src/data/object\_io/mod.rs](/src/data/object_io/mod.rs) | Rust | 1 | 0 | 0 | 1 |
| [src/data\_creater/data\_creater\_impl/batch.rs](/src/data_creater/data_creater_impl/batch.rs) | Rust | 50 | 0 | 7 | 57 |
| [src/data\_creater/data\_creater\_impl/drain.rs](/src/data_creater/data_creater_impl/drain.rs) | Rust | 57 | 0 | 18 | 75 |
| [src/data\_creater/data\_creater\_impl/mod.rs](/src/data_creater/data_creater_impl/mod.rs) | Rust | 120 | 0 | 18 | 138 |
| [src/data\_creater/mod.rs](/src/data_creater/mod.rs) | Rust | 2 | 0 | 2 | 4 |
| [src/error.rs](/src/error.rs) | Rust | 22 | 0 | 8 | 30 |
| [src/excel\_reader.rs](/src/excel_reader.rs) | Rust | 92 | 1 | 19 | 112 |
| [src/extractor\_pipeline.rs](/src/extractor_pipeline.rs) | Rust | 2 | 0 | 1 | 3 |
| [src/extractor\_pipeline/core/mod.rs](/src/extractor_pipeline/core/mod.rs) | Rust | 87 | 0 | 19 | 106 |
| [src/extractor\_pipeline/core/reader.rs](/src/extractor_pipeline/core/reader.rs) | Rust | 19 | 0 | 4 | 23 |
| [src/extractor\_pipeline/core/sink.rs](/src/extractor_pipeline/core/sink.rs) | Rust | 29 | 0 | 8 | 37 |
| [src/extractor\_pipeline/core/worker.rs](/src/extractor_pipeline/core/worker.rs) | Rust | 71 | 0 | 8 | 79 |
| [src/main.rs](/src/main.rs) | Rust | 133 | 0 | 19 | 152 |
| [src/router/mod.rs](/src/router/mod.rs) | Rust | 83 | 0 | 16 | 99 |
| [src/router/route.rs](/src/router/route.rs) | Rust | 15 | 0 | 4 | 19 |
| [src/router/toml\_reader.rs](/src/router/toml_reader.rs) | Rust | 19 | 0 | 5 | 24 |
| [src/sink/data\_checker.rs](/src/sink/data_checker.rs) | Rust | 82 | 0 | 11 | 93 |
| [src/sink/data\_sink.rs](/src/sink/data_sink.rs) | Rust | 14 | 0 | 6 | 20 |
| [src/sink/database/invalid\_sqlite\_sink.rs](/src/sink/database/invalid_sqlite_sink.rs) | Rust | 50 | 0 | 11 | 61 |
| [src/sink/database/mod.rs](/src/sink/database/mod.rs) | Rust | 5 | 0 | 1 | 6 |
| [src/sink/database/sqlite\_database\_sink.rs](/src/sink/database/sqlite_database_sink.rs) | Rust | 2 | 0 | 1 | 3 |
| [src/sink/database/sqlite\_sink/config.rs](/src/sink/database/sqlite_sink/config.rs) | Rust | 69 | 0 | 11 | 80 |
| [src/sink/database/sqlite\_sink/flush.rs](/src/sink/database/sqlite_sink/flush.rs) | Rust | 101 | 0 | 13 | 114 |
| [src/sink/database/sqlite\_sink/mod.rs](/src/sink/database/sqlite_sink/mod.rs) | Rust | 179 | 0 | 20 | 199 |
| [src/sink/database/sqlite\_writer.rs](/src/sink/database/sqlite_writer.rs) | Rust | 131 | 0 | 17 | 148 |
| [src/sink/database/valid\_sqlite\_sink.rs](/src/sink/database/valid_sqlite_sink.rs) | Rust | 50 | 0 | 11 | 61 |
| [src/sink/end\_sink.rs](/src/sink/end_sink.rs) | Rust | 17 | 0 | 6 | 23 |
| [src/sink/material\_production/material\_production\_sink.rs](/src/sink/material_production/material_production_sink.rs) | Rust | 133 | 0 | 21 | 154 |
| [src/sink/material\_production/material\_production\_stat.rs](/src/sink/material_production/material_production_stat.rs) | Rust | 6 | 0 | 2 | 8 |
| [src/sink/material\_production/mod.rs](/src/sink/material_production/mod.rs) | Rust | 4 | 0 | 2 | 6 |
| [src/sink/mod.rs](/src/sink/mod.rs) | Rust | 10 | 0 | 0 | 10 |
| [src/sink/operator\_puaration/mod.rs](/src/sink/operator_puaration/mod.rs) | Rust | 10 | 0 | 3 | 13 |
| [src/sink/operator\_puaration/operator\_output\_metric\_columns.rs](/src/sink/operator_puaration/operator_output_metric_columns.rs) | Rust | 9 | 0 | 2 | 11 |
| [src/sink/operator\_puaration/operator\_parser.rs](/src/sink/operator_puaration/operator_parser.rs) | Rust | 29 | 0 | 8 | 37 |
| [src/sink/operator\_puaration/operator\_puaration\_sink.rs](/src/sink/operator_puaration/operator_puaration_sink.rs) | Rust | 1 | 0 | 1 | 2 |
| [src/sink/operator\_puaration/operator\_puaration\_sink\_config.rs](/src/sink/operator_puaration/operator_puaration_sink_config.rs) | Rust | 66 | 0 | 4 | 70 |
| [src/sink/operator\_puaration/operator\_puaration\_stat.rs](/src/sink/operator_puaration/operator_puaration_stat.rs) | Rust | 8 | 0 | 1 | 9 |
| [src/sink/operator\_puaration/sink/mod.rs](/src/sink/operator_puaration/sink/mod.rs) | Rust | 87 | 0 | 12 | 99 |
| [src/sink/operator\_puaration/sink/persist.rs](/src/sink/operator_puaration/sink/persist.rs) | Rust | 95 | 0 | 15 | 110 |
| [src/sink/operator\_puaration/sink/stats.rs](/src/sink/operator_puaration/sink/stats.rs) | Rust | 65 | 0 | 11 | 76 |
| [src/sink/puaration/mod.rs](/src/sink/puaration/mod.rs) | Rust | 9 | 0 | 3 | 12 |
| [src/sink/puaration/output\_metric\_columns.rs](/src/sink/puaration/output_metric_columns.rs) | Rust | 11 | 0 | 2 | 13 |
| [src/sink/puaration/process\_vector.rs](/src/sink/puaration/process_vector.rs) | Rust | 9 | 0 | 2 | 11 |
| [src/sink/puaration/puaration\_sink/mod.rs](/src/sink/puaration/puaration_sink/mod.rs) | Rust | 173 | 0 | 22 | 195 |
| [src/sink/puaration/puaration\_sink/persist/main.rs](/src/sink/puaration/puaration_sink/persist/main.rs) | Rust | 140 | 0 | 18 | 158 |
| [src/sink/puaration/puaration\_sink/persist/mod.rs](/src/sink/puaration/puaration_sink/persist/mod.rs) | Rust | 2 | 0 | 1 | 3 |
| [src/sink/puaration/puaration\_sink/persist/top\_n.rs](/src/sink/puaration/puaration_sink/persist/top_n.rs) | Rust | 71 | 0 | 10 | 81 |
| [src/sink/puaration/puaration\_sink/stats.rs](/src/sink/puaration/puaration_sink/stats.rs) | Rust | 88 | 0 | 13 | 101 |
| [src/sink/puaration/puaration\_sink\_config.rs](/src/sink/puaration/puaration_sink_config.rs) | Rust | 70 | 0 | 4 | 74 |
| [src/sink/puaration/puaration\_stat.rs](/src/sink/puaration/puaration_stat.rs) | Rust | 46 | 0 | 4 | 50 |
| [src/sink/puaration/vector\_field\_config.rs](/src/sink/puaration/vector_field_config.rs) | Rust | 6 | 0 | 2 | 8 |
| [src/sink/quality\_inspection/mod.rs](/src/sink/quality_inspection/mod.rs) | Rust | 6 | 0 | 2 | 8 |
| [src/sink/quality\_inspection/quality\_param.rs](/src/sink/quality_inspection/quality_param.rs) | Rust | 92 | 0 | 13 | 105 |
| [src/sink/quality\_inspection/quality\_stat.rs](/src/sink/quality_inspection/quality_stat.rs) | Rust | 38 | 0 | 4 | 42 |
| [src/sink/quality\_inspection/sink/mod.rs](/src/sink/quality_inspection/sink/mod.rs) | Rust | 110 | 0 | 22 | 132 |
| [src/sink/quality\_inspection/sink/persist.rs](/src/sink/quality_inspection/sink/persist.rs) | Rust | 105 | 0 | 13 | 118 |
| [src/sink/quality\_inspection/sink/process.rs](/src/sink/quality_inspection/sink/process.rs) | Rust | 154 | 0 | 23 | 177 |
| [src/sink/regist\_info.rs](/src/sink/regist_info.rs) | Rust | 5 | 0 | 1 | 6 |
| [src/sink/stats\_flow/mod.rs](/src/sink/stats_flow/mod.rs) | Rust | 4 | 0 | 3 | 7 |
| [src/sink/stats\_flow/stats\_flow\_config.rs](/src/sink/stats_flow/stats_flow_config.rs) | Rust | 17 | 0 | 3 | 20 |
| [src/sink/stats\_flow/stats\_flow\_sink.rs](/src/sink/stats_flow/stats_flow_sink.rs) | Rust | 117 | 0 | 19 | 136 |
| [src/test/data\_checker\_test.rs](/src/test/data_checker_test.rs) | Rust | 62 | 0 | 15 | 77 |
| [src/test/extractor\_pipeline/forward\_sink.rs](/src/test/extractor_pipeline/forward_sink.rs) | Rust | 28 | 0 | 6 | 34 |
| [src/test/extractor\_pipeline/leaf\_sink.rs](/src/test/extractor_pipeline/leaf_sink.rs) | Rust | 25 | 0 | 6 | 31 |
| [src/test/extractor\_pipeline/mock\_creater.rs](/src/test/extractor_pipeline/mock_creater.rs) | Rust | 46 | 0 | 8 | 54 |
| [src/test/extractor\_pipeline/mock\_reader.rs](/src/test/extractor_pipeline/mock_reader.rs) | Rust | 16 | 0 | 4 | 20 |
| [src/test/extractor\_pipeline/mod.rs](/src/test/extractor_pipeline/mod.rs) | Rust | 5 | 0 | 1 | 6 |
| [src/test/extractor\_pipeline/run\_should\_send\_all\_drain\_data\_to\_sink\_chain\_test.rs](/src/test/extractor_pipeline/run_should_send_all_drain_data_to_sink_chain_test.rs) | Rust | 26 | 0 | 5 | 31 |
| [src/test/material\_production\_sink\_test.rs](/src/test/material_production_sink_test.rs) | Rust | 34 | 0 | 9 | 43 |
| [src/test/mod.rs](/src/test/mod.rs) | Rust | 9 | 0 | 1 | 10 |
| [src/test/operator\_parser\_test.rs](/src/test/operator_parser_test.rs) | Rust | 18 | 0 | 4 | 22 |
| [src/test/operator\_puaration\_sink\_test.rs](/src/test/operator_puaration_sink_test.rs) | Rust | 71 | 0 | 10 | 81 |
| [src/test/puaration\_sink\_test.rs](/src/test/puaration_sink_test.rs) | Rust | 223 | 5 | 34 | 262 |
| [src/test/quality\_inspection\_sink\_test.rs](/src/test/quality_inspection_sink_test.rs) | Rust | 122 | 1 | 33 | 156 |
| [src/test/quality\_param\_test.rs](/src/test/quality_param_test.rs) | Rust | 152 | 0 | 19 | 171 |
| [src/test/sqlite\_database\_sink\_test.rs](/src/test/sqlite_database_sink_test.rs) | Rust | 78 | 0 | 14 | 92 |
| [src/traits.rs](/src/traits.rs) | Rust | 11 | 0 | 3 | 14 |

[Summary](results.md) / Details / [Diff Summary](diff.md) / [Diff Details](diff-details.md)