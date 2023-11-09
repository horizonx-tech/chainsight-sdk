mod sources_algorithm_indexer {
    use std::collections::HashMap;

    use chainsight_cdk::core::{Sources, SourceType};
    use chainsight_cdk_macros::algorithm_indexer_source;

    fn get_target_addr() -> String {
        "target_addr".to_string()
    }
    fn get_indexing_interval() -> u32 {
        60*60
    }
    algorithm_indexer_source!();

    #[test]
    fn test_get_sources() {
        let sources: Vec<Sources<HashMap<String, String>>> = get_sources();
        assert_eq!(sources.len(), 1);
        let source = sources.first().unwrap();
        assert_eq!(source.source_type, SourceType::Chainsight);
        assert_eq!(source.source, get_target_addr());
        assert_eq!(source.attributes, HashMap::new());
        assert_eq!(source.interval_sec, Some(60*60));
    }
}