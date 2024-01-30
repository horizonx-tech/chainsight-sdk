mod test_init_in_env {
    use candid::{Decode, Encode};
    use chainsight_cdk_macros::{init_in, manage_single_state, StableMemoryStorable};

    init_in!();

    #[test]
    fn test() {
        let before = get_initializing_state();
        assert_eq!(before, InitializingState::default());
        assert_eq!(is_initialized(), before.initialized);
        assert_eq!(get_env(), before.env);

        let updated = InitializingState {
            initialized: true,
            proxy: candid::Principal::anonymous().to_text(),
            env: chainsight_cdk::core::Env::Production,
        };
        set_initializing_state(updated.clone());

        assert_eq!(get_initializing_state(), updated);
        assert_eq!(
            proxy(),
            candid::Principal::from_text(&updated.proxy).unwrap()
        );
        assert_eq!(
            get_proxy(),
            candid::Principal::from_text(&updated.proxy).unwrap()
        );
        assert_eq!(is_initialized(), updated.initialized);
        assert_eq!(get_env(), updated.env);
    }
}
