mod sources_snapshot_indexer_web3 {
    use chainsight_cdk::{core::{Sources, SourceType, Web3SnapshotIndexerSourceAttrs, Env}, web3::Web3CtxParam};
    use chainsight_cdk_macros::snapshot_indexer_web3_source;

    fn get_target_addr() -> String {
        "target_addr".to_string()
    }
    fn get_indexing_interval() -> u32 {
        60*60
    }
    fn get_web3_ctx_param() -> Web3CtxParam {
        Web3CtxParam {
            url: "".to_string(),
            from: None,
            chain_id: 31337,
            env: Env::LocalDevelopment,
        }
    }
    snapshot_indexer_web3_source!("total_supply");

    #[test]
    fn test_get_sources() {
        let sources: Vec<Sources<Web3SnapshotIndexerSourceAttrs>> = get_sources();
        assert_eq!(sources.len(), 1);
        let source = sources.first().unwrap();
        assert_eq!(source.source_type, SourceType::Evm);
        assert_eq!(source.source, get_target_addr());
        assert_eq!(source.attributes, Web3SnapshotIndexerSourceAttrs {
            chain_id: 31337,
            function_name: "total_supply".to_string(),
        });
        assert_eq!(source.interval_sec, Some(60*60));
    }
}