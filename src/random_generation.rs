extern crate rand_distr;
extern crate rand;

use petgraph::graph::NodeIndex;
use rand::{rngs::StdRng, Rng, SeedableRng};
use rand_distr::{Distribution, Normal};
use crate::na;

pub fn generate_random_time_exec(device_num:usize, seed:u64, mean:f64, std_dev:f64 ) -> na::DVector<f64>{

    let mut rng = StdRng::seed_from_u64(seed);
    let random_time_generator = Normal::<f64>::new(mean, std_dev).unwrap();

    let mut time_exec = na::DVector::<f64>::zeros(device_num);

    for i in 0..device_num {
        let mut time  = random_time_generator.sample(&mut rng);
        //when generated number is minus, filp its value to the value plus mean
        if time < 0.0 {
            time = time.abs() + mean + mean;
        }
        time_exec[i] = time;
    }
    time_exec

}

pub fn generate_random_edge_distribution(router_num:usize,device_num:usize,seed:u64) -> na::DMatrix<i32>{
    
    let mut edge_distribution = na::DMatrix::<i32>::from_element(router_num, device_num, 0);
    let mut rng = StdRng::seed_from_u64(seed);

    for di in 0..device_num {
        let r_index = rng.random_range(0..router_num);

        for ri in 0..router_num{
           edge_distribution[(ri,di)] =
               if ri == r_index { 1 }
               else { 0 };
        }
    }
    edge_distribution
}

pub fn random_locate_ps(router_num:usize,ps:&NodeIndex, r: &Vec<NodeIndex>, seed:u64) 
    -> (NodeIndex, NodeIndex){

    let mut rng = StdRng::seed_from_u64(seed);
    let node_index = rng.random_range(0..router_num);
    println!("PS allocated router {}",node_index);

    let ps_allocated_router = r[node_index];

    (*ps,ps_allocated_router)

}

//pub fn generate_random_topology()
