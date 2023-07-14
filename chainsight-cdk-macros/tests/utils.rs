mod test_monitoring_canister_metrics {
    use chainsight_cdk_macros::{chainsight_common, init_in};
    init_in!();
    chainsight_common!(60);

    #[test]
    fn test() {
        let datum = CanisterMetricsSnapshot {
            timestamp: 1,
            cycles: 100,
        };
        add_canister_metrics_snapshot(datum.clone());
        assert_eq!(metric(), datum.clone());
        assert_eq!(metrics(1), vec![datum.clone()]);
    }
}
