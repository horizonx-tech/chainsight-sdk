#[allow(unused_must_use)]
mod test_init_in_env_with_stable_memory {
    use candid::{Decode, Encode};
    use chainsight_cdk_macros::{
        init_in, prepare_stable_structure, stable_memory_for_scalar, StableMemoryStorable,
    };

    prepare_stable_structure!();
    init_in!(1);

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
        assert!(set_initializing_state_internal(updated.clone()).is_ok());

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
