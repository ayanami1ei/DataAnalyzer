# Details

Date : 2026-04-15 16:39:34

Directory /home/ayanami/桌面/软件设计/DataAnalyzer/src

Total : 71 files,  3647 codes, 31 comments, 633 blanks, all 4311 lines

[Summary](results.md) / Details / [Diff Summary](diff.md) / [Diff Details](diff-details.md)

## Files
| filename | language | code | comment | blank | total |
| :--- | :--- | ---: | ---: | ---: | ---: |
| [src/checker/data\_check\_rule.rs](/src/checker/data_check_rule.rs) | Rust | 4 | 0 | 2 | 6 |
| [src/checker/geometry\_check\_rule.rs](/src/checker/geometry_check_rule.rs) | Rust | 40 | 1 | 8 | 49 |
| [src/checker/mod.rs](/src/checker/mod.rs) | Rust | 2 | 0 | 1 | 3 |
| [src/data/data.rs](/src/data/data.rs) | Rust | 206 | 2 | 40 | 248 |
| [src/data/data\_pair.rs](/src/data/data_pair.rs) | Rust | 9 | 0 | 3 | 12 |
| [src/data/mod.rs](/src/data/mod.rs) | Rust | 3 | 1 | 0 | 4 |
| [src/data/object\_io/config\_reader.rs](/src/data/object_io/config_reader.rs) | Rust | 22 | 0 | 7 | 29 |
| [src/data/object\_io/config\_writer.rs](/src/data/object_io/config_writer.rs) | Rust | 24 | 0 | 6 | 30 |
| [src/data/object\_io/mod.rs](/src/data/object_io/mod.rs) | Rust | 2 | 0 | 0 | 2 |
| [src/data\_creater.rs](/src/data_creater.rs) | Rust | 175 | 1 | 30 | 206 |
| [src/error.rs](/src/error.rs) | Rust | 24 | 0 | 8 | 32 |
| [src/excel\_reader.rs](/src/excel_reader.rs) | Rust | 83 | 1 | 20 | 104 |
| [src/extractor\_pipeline.rs](/src/extractor_pipeline.rs) | Rust | 166 | 6 | 32 | 204 |
| [src/log.rs](/src/log.rs) | Rust | 70 | 0 | 10 | 80 |
| [src/main.rs](/src/main.rs) | Rust | 91 | 18 | 7 | 116 |
| [src/progress\_table.rs](/src/progress_table.rs) | Rust | 43 | 0 | 8 | 51 |
| [src/sink/data\_checker.rs](/src/sink/data_checker.rs) | Rust | 43 | 0 | 10 | 53 |
| [src/sink/data\_sink.rs](/src/sink/data_sink.rs) | Rust | 13 | 0 | 4 | 17 |
| [src/sink/database/database\_row\_mapping.rs](/src/sink/database/database_row_mapping.rs) | Rust | 6 | 0 | 2 | 8 |
| [src/sink/database/mod.rs](/src/sink/database/mod.rs) | Rust | 3 | 0 | 1 | 4 |
| [src/sink/database/mysql\_database\_sink.rs](/src/sink/database/mysql_database_sink.rs) | Rust | 230 | 0 | 36 | 266 |
| [src/sink/database/sqlite\_database\_sink.rs](/src/sink/database/sqlite_database_sink.rs) | Rust | 230 | 0 | 34 | 264 |
| [src/sink/end\_sink.rs](/src/sink/end_sink.rs) | Rust | 11 | 0 | 5 | 16 |
| [src/sink/material\_production/material\_production\_sink.rs](/src/sink/material_production/material_production_sink.rs) | Rust | 128 | 0 | 26 | 154 |
| [src/sink/material\_production/material\_production\_stat.rs](/src/sink/material_production/material_production_stat.rs) | Rust | 6 | 0 | 2 | 8 |
| [src/sink/material\_production/mod.rs](/src/sink/material_production/mod.rs) | Rust | 4 | 0 | 2 | 6 |
| [src/sink/mod.rs](/src/sink/mod.rs) | Rust | 8 | 0 | 1 | 9 |
| [src/sink/operator\_puaration/mod.rs](/src/sink/operator_puaration/mod.rs) | Rust | 11 | 0 | 3 | 14 |
| [src/sink/operator\_puaration/operator\_output\_metric\_columns.rs](/src/sink/operator_puaration/operator_output_metric_columns.rs) | Rust | 9 | 0 | 2 | 11 |
| [src/sink/operator\_puaration/operator\_parser.rs](/src/sink/operator_puaration/operator_parser.rs) | Rust | 29 | 0 | 8 | 37 |
| [src/sink/operator\_puaration/operator\_puaration\_db\_config.rs](/src/sink/operator_puaration/operator_puaration_db_config.rs) | Rust | 7 | 0 | 2 | 9 |
| [src/sink/operator\_puaration/operator\_puaration\_sink.rs](/src/sink/operator_puaration/operator_puaration_sink.rs) | Rust | 268 | 0 | 46 | 314 |
| [src/sink/operator\_puaration/operator\_puaration\_sink\_config.rs](/src/sink/operator_puaration/operator_puaration_sink_config.rs) | Rust | 70 | 0 | 4 | 74 |
| [src/sink/operator\_puaration/operator\_puaration\_stat.rs](/src/sink/operator_puaration/operator_puaration_stat.rs) | Rust | 9 | 0 | 2 | 11 |
| [src/sink/operator\_puaration/operator\_vector\_field\_config.rs](/src/sink/operator_puaration/operator_vector_field_config.rs) | Rust | 6 | 0 | 2 | 8 |
| [src/sink/puaration/mod.rs](/src/sink/puaration/mod.rs) | Rust | 10 | 0 | 3 | 13 |
| [src/sink/puaration/output\_metric\_columns.rs](/src/sink/puaration/output_metric_columns.rs) | Rust | 9 | 0 | 2 | 11 |
| [src/sink/puaration/process\_vector.rs](/src/sink/puaration/process_vector.rs) | Rust | 9 | 0 | 2 | 11 |
| [src/sink/puaration/puaration\_db\_config.rs](/src/sink/puaration/puaration_db_config.rs) | Rust | 7 | 0 | 2 | 9 |
| [src/sink/puaration/puaration\_sink/db\_persistence.rs](/src/sink/puaration/puaration_sink/db_persistence.rs) | Rust | 144 | 0 | 23 | 167 |
| [src/sink/puaration/puaration\_sink/json\_persistence.rs](/src/sink/puaration/puaration_sink/json_persistence.rs) | Rust | 80 | 0 | 12 | 92 |
| [src/sink/puaration/puaration\_sink/mod.rs](/src/sink/puaration/puaration_sink/mod.rs) | Rust | 58 | 0 | 12 | 70 |
| [src/sink/puaration/puaration\_sink/stats.rs](/src/sink/puaration/puaration_sink/stats.rs) | Rust | 64 | 0 | 13 | 77 |
| [src/sink/puaration/puaration\_sink\_config.rs](/src/sink/puaration/puaration_sink_config.rs) | Rust | 68 | 0 | 4 | 72 |
| [src/sink/puaration/puaration\_stat.rs](/src/sink/puaration/puaration_stat.rs) | Rust | 8 | 0 | 1 | 9 |
| [src/sink/puaration/vector\_field\_config.rs](/src/sink/puaration/vector_field_config.rs) | Rust | 6 | 0 | 2 | 8 |
| [src/sink/stats\_flow/mod.rs](/src/sink/stats_flow/mod.rs) | Rust | 4 | 0 | 3 | 7 |
| [src/sink/stats\_flow/stats\_flow\_config.rs](/src/sink/stats_flow/stats_flow_config.rs) | Rust | 16 | 0 | 3 | 19 |
| [src/sink/stats\_flow/stats\_flow\_sink.rs](/src/sink/stats_flow/stats_flow_sink.rs) | Rust | 65 | 0 | 12 | 77 |
| [src/test/data\_checker\_test.rs](/src/test/data_checker_test.rs) | Rust | 53 | 0 | 15 | 68 |
| [src/test/data\_creater\_test.rs](/src/test/data_creater_test.rs) | Rust | 356 | 0 | 41 | 397 |
| [src/test/db\_fetcher.rs](/src/test/db_fetcher.rs) | Rust | 53 | 0 | 8 | 61 |
| [src/test/error\_checker.rs](/src/test/error_checker.rs) | Rust | 16 | 0 | 3 | 19 |
| [src/test/extractor\_pipeline/forward\_sink.rs](/src/test/extractor_pipeline/forward_sink.rs) | Rust | 20 | 0 | 5 | 25 |
| [src/test/extractor\_pipeline/leaf\_sink.rs](/src/test/extractor_pipeline/leaf_sink.rs) | Rust | 18 | 0 | 5 | 23 |
| [src/test/extractor\_pipeline/mock\_creater.rs](/src/test/extractor_pipeline/mock_creater.rs) | Rust | 35 | 0 | 8 | 43 |
| [src/test/extractor\_pipeline/mock\_reader.rs](/src/test/extractor_pipeline/mock_reader.rs) | Rust | 16 | 0 | 4 | 20 |
| [src/test/extractor\_pipeline/mod.rs](/src/test/extractor_pipeline/mod.rs) | Rust | 5 | 0 | 1 | 6 |
| [src/test/extractor\_pipeline/run\_should\_send\_all\_drain\_data\_to\_sink\_chain\_test.rs](/src/test/extractor_pipeline/run_should_send_all_drain_data_to_sink_chain_test.rs) | Rust | 31 | 0 | 5 | 36 |
| [src/test/geometry\_error\_entry.rs](/src/test/geometry_error_entry.rs) | Rust | 30 | 0 | 5 | 35 |
| [src/test/log\_check\_validator.rs](/src/test/log_check_validator.rs) | Rust | 46 | 1 | 7 | 54 |
| [src/test/log\_entry.rs](/src/test/log_entry.rs) | Rust | 6 | 0 | 1 | 7 |
| [src/test/log\_parser.rs](/src/test/log_parser.rs) | Rust | 18 | 0 | 3 | 21 |
| [src/test/material\_production\_sink\_test.rs](/src/test/material_production_sink_test.rs) | Rust | 39 | 0 | 8 | 47 |
| [src/test/mod.rs](/src/test/mod.rs) | Rust | 15 | 0 | 1 | 16 |
| [src/test/operator\_parser\_test.rs](/src/test/operator_parser_test.rs) | Rust | 18 | 0 | 4 | 22 |
| [src/test/operator\_puaration\_sink\_test.rs](/src/test/operator_puaration_sink_test.rs) | Rust | 59 | 0 | 9 | 68 |
| [src/test/puaration\_sink\_test.rs](/src/test/puaration_sink_test.rs) | Rust | 70 | 0 | 12 | 82 |
| [src/test/sqlite\_database\_sink\_test.rs](/src/test/sqlite_database_sink_test.rs) | Rust | 69 | 0 | 11 | 80 |
| [src/test/test\_log\_check.rs](/src/test/test_log_check.rs) | Rust | 60 | 0 | 6 | 66 |
| [src/traits.rs](/src/traits.rs) | Rust | 11 | 0 | 3 | 14 |

[Summary](results.md) / Details / [Diff Summary](diff.md) / [Diff Details](diff-details.md)