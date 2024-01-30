mod test_define_web3_ctx_with_stable_memory {
    use chainsight_cdk_macros::{
        define_web3_ctx, prepare_stable_structure, stable_memory_for_scalar,
    };
    use ic_web3_rs::types::Address;
    use std::str::FromStr;

    prepare_stable_structure!();
    define_web3_ctx!(1);

    #[test]
    fn test() {
        assert_eq!(
            get_web3_ctx_param(),
            chainsight_cdk::web3::Web3CtxParam::default()
        );

        let ctx_param = chainsight_cdk::web3::Web3CtxParam {
            url: "http://localhost:8545".to_string(),
            from: Some("0x0000000000000000000000000000000000000000".to_string()),
            chain_id: 1,
            env: chainsight_cdk::core::Env::Production,
        };
        assert!(set_web3_ctx_param(ctx_param.clone()).is_ok());
        assert_eq!(get_web3_ctx_param(), ctx_param);

        let ctx = web3_ctx().unwrap();
        assert_eq!(
            ctx.from(),
            Address::from_str(&ctx_param.from.unwrap()).unwrap()
        );
        assert_eq!(ctx.chain_id(), ctx_param.chain_id);
        assert_eq!(ctx.key_name(), ctx_param.env.ecdsa_key_name());
    }
}
