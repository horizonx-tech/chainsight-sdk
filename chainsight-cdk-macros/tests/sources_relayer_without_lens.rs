mod sources_relayer_without_lens {
    use chainsight_cdk::core::{RelayerWithLensSourceAttrs, SourceType, Sources};
    use chainsight_cdk_macros::relayer_source;

    fn get_target_canister() -> String {
        "target_canister".to_string()
    }
    fn get_indexing_interval() -> u32 {
        60 * 60
    }
    relayer_source!("icrc1_balance_of", false);

    #[test]
    fn test_get_sources() {
        let sources: Vec<Sources<RelayerWithLensSourceAttrs>> = get_sources();
        assert_eq!(sources.len(), 1);
        let source = sources.first().unwrap();
        assert_eq!(source.source_type, SourceType::Chainsight);
        assert_eq!(source.source, get_target_canister());
        assert_eq!(
            source.attributes,
            RelayerWithLensSourceAttrs {
                function_name: "icrc1_balance_of".to_string(),
                sources: vec![]
            }
        );
        assert_eq!(source.interval_sec, Some(60 * 60));
    }
}
