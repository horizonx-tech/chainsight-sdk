mod sources_relayer_with_lens {
    use chainsight_cdk::core::{Sources, SourceType, RelayerWithLensSourceAttrs};
    use chainsight_cdk_macros::relayer_source;

    fn get_target_canister() -> String {
        "target_canister".to_string()
    }
    fn get_indexing_interval() -> u32 {
        60*60
    }
    fn call_args() -> Vec<String> {
        vec![
            "dummy-00000-00000-00000-001".to_string(),
            "dummy-00000-00000-00000-002".to_string(),
            "dummy-00000-00000-00000-003".to_string(),
        ]
    }
    relayer_source!("icrc1_balance_of", true);

    #[test]
    fn test_get_sources() {
        let sources: Vec<Sources<RelayerWithLensSourceAttrs>> = get_sources();
        assert_eq!(sources.len(), 1);
        let source = sources.first().unwrap();
        assert_eq!(source.source_type, SourceType::Chainsight);
        assert_eq!(source.source, get_target_canister());
        assert_eq!(source.attributes, RelayerWithLensSourceAttrs {
            function_name: "icrc1_balance_of".to_string(),
            sources: call_args()
        });
        assert_eq!(source.interval_sec, Some(60*60));
    }
}