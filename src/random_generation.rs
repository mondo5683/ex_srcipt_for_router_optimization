extern crate rand;
extern crate rand_distr;

use crate::na;
use na::DVector;
use petgraph::graph::NodeIndex;
use rand::{rngs::StdRng, Rng, SeedableRng};
use rand_distr::{Distribution, Normal};
use std::collections::HashMap;

pub fn generate_random_time_exec(
    device_num: usize,
    seed: u64,
    mean: f64,
    std_dev: f64,
) -> na::DVector<f64> {
    let mut rng = StdRng::seed_from_u64(seed);
    let random_time_generator = Normal::<f64>::new(mean, std_dev).unwrap();

    let mut time_exec = na::DVector::<f64>::zeros(device_num);

    for i in 0..device_num {
        let mut time = random_time_generator.sample(&mut rng);
        //when generated number is minus, filp its value to the value plus mean
        if time < 0.0 {
            time = time.abs() + mean + mean;
        }
        time_exec[i] = time;
    }
    time_exec
}

pub fn generate_random_time_exec_based_on_ratio(device_num: usize, seed: u64) -> na::DVector<f64> {
    let device_type_order = vec!["server", "desktop", "laptop", "raspi", "sensor"];

    let hashmap_device_type_to_ratio = HashMap::from([
        ("server", 0.1),
        ("desktop", 0.3),
        ("laptop", 0.3),
        ("raspi", 0.2),
        ("sensor", 0.1),
    ]);

    let hashmap_timeexec_kind_to_mean_and_std = HashMap::from([
        ("server", (2.0, 0.3)),
        ("desktop", (40.0, 8.0)),
        ("laptop", (80.0, 12.0)),
        ("raspi", (150.0, 20.0)),
        ("sensor", (400.0, 60.0)),
    ]);

    let mut time_exec_buff = Vec::<f64>::new();
    for device_type in device_type_order {
        let mean_and_std = hashmap_timeexec_kind_to_mean_and_std[device_type];
        let ratio = hashmap_device_type_to_ratio[device_type];
        let mut num_samples = (device_num as f64 * ratio) as usize;

        let time_exec_for_the_device_type =
            generate_random_time_exec(num_samples, seed, mean_and_std.0, mean_and_std.1)
                .iter()
                .copied()
                .collect::<Vec<f64>>();
        time_exec_buff.extend(time_exec_for_the_device_type);
    }

    let time_exec = DVector::from_vec(time_exec_buff.clone());
    time_exec
}

pub fn generate_random_edge_distribution(
    router_num: usize,
    device_num: usize,
    seed: u64,
) -> na::DMatrix<i32> {
    let mut edge_distribution = na::DMatrix::<i32>::from_element(router_num, device_num, 0);
    let mut rng = StdRng::seed_from_u64(seed);

    for di in 0..device_num {
        let r_index = rng.random_range(0..router_num);

        for ri in 0..router_num {
            edge_distribution[(ri, di)] = if ri == r_index { 1 } else { 0 };
        }
    }
    edge_distribution
}

pub fn random_locate_ps(
    router_num: usize,
    ps: &NodeIndex,
    r: &Vec<NodeIndex>,
    seed: u64,
) -> (NodeIndex, NodeIndex) {
    let mut rng = StdRng::seed_from_u64(seed);
    let node_index = rng.random_range(0..router_num);
    println!("PS allocated router {}", node_index);

    let ps_allocated_router = r[node_index];

    (*ps, ps_allocated_router)
}

//pub fn generate_random_topology()
