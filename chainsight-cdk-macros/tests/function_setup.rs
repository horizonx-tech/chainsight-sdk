mod test_setup {
    use chainsight_cdk_macros::{manage_single_state, setup_func};

    manage_single_state!("rpc", String, false);
    manage_single_state!("chain_id", u8, false);
    manage_single_state!("dst_address", String, false);
    setup_func!({
        rpc: String,
        chain_id: u8,
        dst_address: String,
    });

    #[test]
    #[should_panic(expected = "Already setup")]
    fn test() {
        let rpc = String::from("rpc");
        let chain_id = 1;
        let dst_address = String::from("dst_address");

        assert_eq!(get_setup_flag(), false);
        assert!(setup(rpc.clone(), chain_id, dst_address.clone()).is_ok());
        assert_eq!(get_rpc(), rpc);
        assert_eq!(get_chain_id(), chain_id);
        assert_eq!(get_dst_address(), dst_address);
        assert_eq!(get_setup_flag(), true);

        let _ = setup(rpc.clone(), chain_id, dst_address.clone());
    }
}
