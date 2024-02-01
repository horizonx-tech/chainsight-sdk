#[allow(unused_must_use)]
mod function_setup_with_stable_memory {
    use chainsight_cdk::storage::StorableStrings;
    use chainsight_cdk_macros::{prepare_stable_structure, setup_func, stable_memory_for_scalar};

    stable_memory_for_scalar!("rpc", String, 1, false);
    stable_memory_for_scalar!("chain_id", u8, 2, false);
    stable_memory_for_scalar!("dst_address", String, 3, false);
    stable_memory_for_scalar!("lens_targets", StorableStrings, 4, false);

    prepare_stable_structure!();

    setup_func!({
        rpc: String,
        chain_id: u8,
        dst_address: String,
        lens_targets: Vec<String>
    }, 5);

    #[test]
    #[should_panic(expected = "Already setup")]
    fn test() {
        let rpc = String::from("rpc");
        let chain_id = 1;
        let dst_address = String::from("dst_address");
        let lens_targets = vec![
            "target_1".to_string(),
            "target_2".to_string(),
            "target_3".to_string(),
        ];

        assert_eq!(get_setup_flag(), false.into());
        assert!(setup(
            rpc.clone(),
            chain_id,
            dst_address.clone(),
            lens_targets.clone()
        )
        .is_ok());
        assert_eq!(get_rpc(), rpc);
        assert_eq!(get_chain_id(), chain_id);
        assert_eq!(get_dst_address(), dst_address);
        assert_eq!(get_lens_targets(), StorableStrings(lens_targets.clone()));
        assert_eq!(get_setup_flag(), true.into());

        let _ = setup(rpc.clone(), chain_id, dst_address.clone(), lens_targets);
    }
}
