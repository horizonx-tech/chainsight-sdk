use chainsight_cdk_macros::{manage_single_state, setup_func, manage_vec_state, timer_task_func, manage_map_state, monitoring_canister_metrics, did_export};

monitoring_canister_metrics!(60);
#[derive(Default, Clone, Debug, PartialEq, candid::CandidType, candid::Deserialize)]
pub struct Parameter {
    pub a: u64,
    pub b: u64,
}
manage_single_state!("parameter", Parameter, false);
setup_func!({
    parameter: Parameter,
});
manage_single_state!("x", u64, false);
manage_vec_state!("solution", u64, true);
manage_vec_state!("solution_ts", u64, true);
manage_map_state!("map_solution", u64, u64, false);
static LINEAR_EQUATION: fn() -> () = || {
    let current_ts_sec = ic_cdk::api::time() / 1000000;
    let param = get_parameter();
    let x = get_x();
    let solution = param.a * x + param.b;
    add_solution(solution);
    add_solution_ts(current_ts_sec);
    insert_map_solution(current_ts_sec, solution);
    ic_cdk::println!("x={}, solution={}, ts={}", x, solution, current_ts_sec);
    set_x(x + 1);
};
timer_task_func!("set_task", "LINEAR_EQUATION", false);

did_export!("interface");
