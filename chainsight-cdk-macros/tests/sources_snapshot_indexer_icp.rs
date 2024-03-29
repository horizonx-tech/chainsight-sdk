mod sources_snapshot_indexer_web3 {
    use chainsight_cdk::core::{ICSnapshotIndexerSourceAttrs, SourceType, Sources};
    use chainsight_cdk_macros::snapshot_indexer_icp_source;

    fn get_target_canister() -> String {
        "target_canister".to_string()
    }
    fn get_indexing_interval() -> u32 {
        60 * 60
    }
    snapshot_indexer_icp_source!("icrc1_balance_of");

    #[test]
    fn test_get_sources() {
        let sources: Vec<Sources<ICSnapshotIndexerSourceAttrs>> = get_sources();
        assert_eq!(sources.len(), 1);
        let source = sources.first().unwrap();
        assert_eq!(source.source_type, SourceType::Chainsight);
        assert_eq!(source.source, get_target_canister());
        assert_eq!(
            source.attributes,
            ICSnapshotIndexerSourceAttrs {
                function_name: "icrc1_balance_of".to_string(),
                sources: vec![],
            }
        );
        assert_eq!(source.interval_sec, Some(60 * 60));
    }
}
