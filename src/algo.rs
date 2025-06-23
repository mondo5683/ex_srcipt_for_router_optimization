extern crate petgraph;
extern crate nalgebra as na;
mod router;
mod graph_operation;
mod sell_info;
mod gen_topology;
mod algorithms;
mod random_generation;

use std::collections::HashMap;

use gen_topology::{generate_device_array, generate_edge_array, generate_router_array};
use petgraph::{algo::all_simple_paths, Graph, Undirected};
use random_generation::{generate_random_edge_distribution, generate_random_time_exec, random_locate_ps};
use router::Router;
use algorithms::optimize_obvalue_in_cluster;

const ALPHA:f64 = 0.5;
const BETA:f64 = 0.5;
const INF:f64 = 9999999999999999.0;

//Parameters for random_generation
const DEVICE_NUM: usize = 5000;
const TE_MEAN : f64 = 32.0;
const TE_STD:f64 = 97.0;
const TE_SEED: u64 = 123;
const ED_SEED: u64 = 123;
const PS_LOCATION_SEED: u64 = 123;


fn main() {

//     let rt = na::dmatrix![
//        0,1,1,1;
//        0,0,0,0;
//        0,0,0,0;
//        0,0,0,0;
//    ];
    let n = 1190; // 総ルータ数
    let mut matrix = na::DMatrix::<i32>::zeros(n, n);

    // ルータの接続関係（最上位14個）
    let edges = vec![
        (0,1), (0,2), (0,7), (1,2), (1,3), (2,5), (3,4), (3,8),
        (4,5), (4,6), (5,9), (5,12), (6,7), (7,10), (8,11), (8,13),
        (10,11), (10,13), (10,9), (11,12), (12,13)
    ];

    for &(a, b) in &edges {
        matrix[(a, b)] = 1;
//        matrix[(b, a)] = 1;
    }

    // 各ルータに4分木の子を4層追加
    let mut next_id = 14; // 13番目まで使っているので14から開始
    let mut current_layer = vec![0,1,2,3,4,5,6,7,8,9,10,11,12,13];
    
    for _ in 0..4 { // 4層分の4分木を作成
        let mut next_layer = Vec::new();
        for &parent in &current_layer {
            for _ in 0..4 {
                if next_id >= n { break; }
                matrix[(parent, next_id)] = 1;
//                matrix[(next_id, parent)] = 1;
                next_layer.push(next_id);
                next_id += 1;
            }
        }
        current_layer = next_layer;
    }

    // 0番目に接続されているルータのみ出力
    println!("Connections of router 0:");
    for i in 0..n {
        if matrix[(i, 1189)] == 1 {
            println!("Router 0 is connected to Router {}", i);
        }
    }
    let rt = matrix;

//    let time_exec = na::vector![20.0,100.0,35.0,100.0,60.0,300.0,40.0];
//    let time_exec = na::vector![20.0,35.0,100.0,100.0,80.0,300.0,40.0];
    let time_exec = generate_random_time_exec(DEVICE_NUM, TE_SEED, TE_MEAN, TE_STD);
    let e = generate_random_edge_distribution(rt.nrows(), DEVICE_NUM, ED_SEED);
//    println!("{:?}", e);
    println!("{:?}",time_exec);

    let mut topology = Graph::<String, String, Undirected>::new_undirected();
    
    let r = generate_router_array(&rt, &mut topology);
    let d = generate_device_array(&e, &mut topology);

    // Create Vertexes
    let ps = topology.add_node(String::from("parameter_server"));
    let mut edge_array = generate_edge_array(&rt, &e, &r, &d);
    edge_array.push(random_locate_ps(rt.nrows(), &ps, &r, PS_LOCATION_SEED));
    topology.extend_with_edges(&edge_array);

    let model_size = 500.0;
//    println!("{:?}",topology);
    let topology_for_routers = topology.clone();


    let time_exec_average = time_exec.iter().sum::<f64>() as f64 / time_exec.len() as f64 ;

    println!("Time execution average: {}", time_exec_average);

    let mut routers = Vec::<Router>::new();
    for router in r.iter(){
        routers.push(Router::new(&topology_for_routers, router,&ps, model_size));
    } 
    //vecに応じてデバイスを割り当てる
    for (col_index, column) in e.column_iter().enumerate() {
        for (row_index, value) in column.iter().enumerate() {
            if *value == 1 {
                let device = (d[col_index], time_exec[col_index]);
                routers[row_index].add_device(device);
            }
        }
    }

    let mut router_num = 0;
    let mut network_traffic_cost = 0.0;
    let mut max_variance = 0.0;
    for r in routers.iter_mut(){
        if r.get_devices().len() > 0{
//            println!("router:{}",r.get_router().index());
            let (network_traffic, variance) = r.update_obvalue();
            network_traffic_cost += network_traffic;
            if max_variance < variance{
                max_variance = variance;
            }

            router_num += 1;
        }
    }
    println!("network_traffic: {}, variance:{}, obvalue: {}",network_traffic_cost,max_variance,ALPHA*network_traffic_cost + BETA*max_variance);
    println!("router num:{}",router_num);
    
    let mut routers_stratum_dict:HashMap<i32,Vec<usize>> = HashMap::new();
    let mut max_depth = 0;
    for r in routers.iter() {
        let depth_of_r = r.hop_to_ps(&ps);
//        println!("router {:?} is {}",r.get_router(), depth_of_r));
        routers_stratum_dict.entry(depth_of_r).or_insert(vec![]).push(r.get_router().index());

        if max_depth < depth_of_r {
            max_depth = depth_of_r
        }
    }
    println!("Max depth : {}", max_depth);
    
    for depth in (1..max_depth).rev() {

        println!("DEPTH {} ----------------",depth);
        for parent_router_index in routers_stratum_dict.get(&depth).unwrap() {
            let mut inner_cluster_routers_indices = vec![*parent_router_index];
            let inner_cluster_router_vector = rt.row(*parent_router_index);
            
            for (i,router_index) in inner_cluster_router_vector.iter().enumerate() {
                if *router_index == 1 {
                    inner_cluster_routers_indices.push(i);
                }
            }
            let mut inner_cluster_routers = Vec::<&mut Router>::new();
            
            for r in routers.iter_mut(){
                let r_index = r.get_router().index();
                if r_index == *parent_router_index {
                    inner_cluster_routers.insert(0, r);
                }
                else if inner_cluster_routers_indices.contains(&r_index){
                    inner_cluster_routers.push(r);
                }
            }

            optimize_obvalue_in_cluster(&mut inner_cluster_routers);
        }
    }
    network_traffic_cost = 0.0;
    max_variance = 0.0;
    router_num = 0;
    for r in routers.iter_mut(){
        if r.get_devices().len() > 0{
//            println!("router:{}",r.get_router().index());
            let (network_traffic, variance) = r.update_obvalue();
            network_traffic_cost += network_traffic;
            if max_variance < variance{
                max_variance = variance;
            }

            router_num += 1;
        }
    }
    println!("network_traffic: {}, variance:{}, obvalue: {}",network_traffic_cost,max_variance,ALPHA*network_traffic_cost + BETA*max_variance);
    println!("router num:{}",router_num);
}
