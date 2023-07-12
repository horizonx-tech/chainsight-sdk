mod test_monitoring_canister_metrics {
    use chainsight_cdk_macros::{init_in, monitoring_canister_metrics};
    init_in!();
    monitoring_canister_metrics!(60);

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
