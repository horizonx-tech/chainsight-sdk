mod test_timer_task_func {
    use candid::{Decode, Encode};
    use chainsight_cdk_macros::{
        init_in, manage_single_state, timer_task_func, StableMemoryStorable,
    };

    #[allow(dead_code)]
    fn hello() {}

    init_in!();
    timer_task_func!("set_task", "hello");

    #[test]
    fn test() {
        assert_eq!(get_indexing_interval(), 0);

        let interval = 1000;
        set_indexing_interval(interval);
        assert_eq!(get_indexing_interval(), interval);
    }
}
