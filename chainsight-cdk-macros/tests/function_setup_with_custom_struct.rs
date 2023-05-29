mod test_setup_with_custom_struct {
    use candid::CandidType;
    use chainsight_cdk_macros::{manage_single_state, setup_func};
    use serde::Deserialize;

    #[derive(Default, Clone, Debug, PartialEq, CandidType, Deserialize)]
    pub struct Settings {
        rpc: String,
        chain_id: u8,
        dst_address: String,
    }
    manage_single_state!("duration", u8, false);
    manage_single_state!("settings", Settings, false);
    setup_func!({
        duration: u8,
        settings: Settings,
    });

    #[test]
    fn test_setup_with_custom_struct() {
        let duration = 10;
        let settings = Settings {
            rpc: String::from("rpc"),
            chain_id: 1,
            dst_address: String::from("dst_address"),
        };
        assert!(setup(duration, settings.clone()).is_ok());
        assert_eq!(get_duration(), duration);
        assert_eq!(get_settings(), settings.clone());
    }
}