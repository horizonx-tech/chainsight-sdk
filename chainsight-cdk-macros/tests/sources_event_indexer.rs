mod sources_event_indexer {
    use chainsight_cdk::{
        core::{Env, SourceType, Sources, Web3EventIndexerSourceAttrs},
        web3::Web3CtxParam,
    };
    use chainsight_cdk_macros::web3_event_indexer_source;

    fn get_target_addr() -> String {
        "target_addr".to_string()
    }
    fn get_indexing_interval() -> u32 {
        60 * 60
    }
    fn get_web3_ctx_param() -> Web3CtxParam {
        Web3CtxParam {
            url: "".to_string(),
            from: None,
            chain_id: 31337,
            env: Env::LocalDevelopment,
        }
    }
    web3_event_indexer_source!(Transfer);

    #[test]
    fn test_get_sources() {
        let sources: Vec<Sources<Web3EventIndexerSourceAttrs>> = get_sources();
        assert_eq!(sources.len(), 1);
        let source = sources.first().unwrap();
        assert_eq!(source.source_type, SourceType::Evm);
        assert_eq!(source.source, get_target_addr());
        assert_eq!(
            source.attributes,
            Web3EventIndexerSourceAttrs {
                chain_id: 31337,
                event_name: "Transfer".to_string(),
            }
        );
        assert_eq!(source.interval_sec, Some(60 * 60));
    }
}
