#[allow(unused_must_use)]
mod test_timer_task_func_with_stable_memory {
    use candid::{Decode, Encode};
    use chainsight_cdk_macros::{
        init_in, manage_single_state, prepare_stable_structure, stable_memory_for_scalar,
        timer_task_func, StableMemoryStorable,
    };

    #[allow(dead_code)]
    fn hello() {}

    init_in!();
    prepare_stable_structure!();

    timer_task_func!("set_task", "hello", 1);

    #[test]
    fn test() {
        assert_eq!(get_indexing_interval(), 0);

        let interval = 1000;
        set_indexing_interval(interval);
        assert_eq!(get_indexing_interval(), interval);
    }
}
