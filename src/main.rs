extern crate petgraph;
extern crate nalgebra as na;
mod router;
mod graph_operation;
mod sell_info;
mod gen_topology;
mod algorithms;
mod random_generation;
mod device_clustering;
mod benchmark_algorithms;
mod device;
mod manual_operation;


use gen_topology::{generate_device_array, generate_edge_array, generate_router_array};
use petgraph::graph::NodeIndex;
use petgraph::{algo::all_simple_paths, Graph, Undirected};
use random_generation::{generate_random_edge_distribution, generate_random_time_exec_based_on_ratio, random_locate_ps};
use router::Router;
use algorithms::optimize_obvalue_in_cluster;
use rand::{rngs::StdRng, Rng, SeedableRng};
use device::Device;
use na::{DVector};
use clustering::*;
use benchmark_algorithms::*;
use manual_operation::*;

//for printing out "topology.json"
use std::fs::File;
use std::io::{Write, Result};

const ALPHA:f64 = 0.5;
const BETA:f64 = 0.5;
const INF:f64 = 9999999999999999.0;

//Parameters for random_generation
const DEVICE_NUM: usize = 130;
const TE_MEAN : f64 = 32.0;
const SEED:u64 = 100;
const TE_STD:f64 = 97.0;
//const TE_SEED: u64 = 44;
//const ED_SEED: u64 = 55;
//const PS_LOCATION_SEED: u64 = 44;


fn main() -> std::io::Result<()> {

    
     let rt = na::dmatrix![
        0,1,1,1;
        0,0,0,0;
        0,0,0,0;
        0,0,0,0;
    ];
    

/*
    let rt = na::dmatrix![
        0,1,1,0,0,0,0,0,0,0,0;
        0,0,0,1,1,0,0,0,0,0,0;
        0,0,0,0,0,1,1,0,0,0,0;
        0,0,0,0,0,0,0,1,1,0,0;
        0,0,0,0,0,0,0,0,0,0,0;
        0,0,0,0,0,0,0,0,0,0,0;
        0,0,0,0,0,0,0,0,0,1,1;
        0,0,0,0,0,0,0,0,0,0,0;
        0,0,0,0,0,0,0,0,0,0,0;
        0,0,0,0,0,0,0,0,0,0,0;
        0,0,0,0,0,0,0,0,0,0,0;
    ];
*/

    let rt = generate_topology();
    let rand_seeds: [u64; 10] = [100,101,102,103,104,105,106,107,108,109];
    //let rand_seeds: [u64; 4] = [100,109,101,108];
//    let k_vec = [1,2,3,4,5,6,7,8,9,10,11,12,13];
//    let k_vec = [9,10,11,12,13];
    let k_vec = [1,2,3,4];
    let model_size = 500.0;

    for rand in rand_seeds{
        println!("Seed: {}", rand);
        for k in k_vec{ println!("For k: {}", k);
    //    let time_exec = na::vector![20.0,100.0,35.0,100.0,60.0,300.0,40.0];

        let mut rng = StdRng::seed_from_u64(rand);
        let te_seed:u64 = rng.random();
        let ed_seed:u64 = rng.random();
        let ps_location_seed:u64 = rng.random();

        let time_exec = generate_random_time_exec_based_on_ratio(DEVICE_NUM, te_seed);
        let e = generate_random_edge_distribution(rt.nrows(), DEVICE_NUM, ed_seed);
    //    println!("{:?}", e);
        println!("{:?}",time_exec);

        let mut topology = Graph::<String, String, Undirected>::new_undirected();
        
        let r = generate_router_array(&rt, &mut topology);
        let d = generate_device_array(&e, &mut topology);


        // Create Vertexes
        let ps = topology.add_node(String::from("parameter_server"));
        let mut edge_array = generate_edge_array(&rt, &e, &r, &d);
        edge_array.push(random_locate_ps(rt.nrows(), &ps, &r, ps_location_seed));
        //edge_array.push((ps, r[143]));
        topology.extend_with_edges(&edge_array);


    //    println!("{:?}",topology);
        let topology_for_routers = topology.clone();

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
      
        print_current_obvalue(&mut routers);
        
        /*
        match manually_operation(&mut routers, &r, &d, time_exec.column(0).iter().map(|x| *x).collect()){
            Ok(_) => println!("Manual Operation completed."),
            Err(msg) => eprintln!("Manual Operation stops due to error {}",msg),

        };
        */
        
        //clustering(k, &d, &time_exec ,&mut routers);
        optimize_obvalue_in_cluster(&mut routers);

        print_current_obvalue(&mut routers);
        println!("---");
//    println!("router num:{}",router_num);
        
        write_topology_json(routers)?;

    
    //The followed } is the end of loop of k
    }

    }
    Ok(())
}


fn generate_topology() -> na::DMatrix<i32> {

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

    matrix
}

fn print_current_obvalue(routers: &mut Vec<Router>) {

    let mut network_traffic_cost = 0.0;
    let mut max_variance = 0.0;
    for r in routers.iter_mut(){
        if r.get_devices().len() > 0{
//                println!("router:{}",r.get_router().index());
//                println!("router:{:?}",r.get_devices());
        
            let (network_traffic, variance) = r.update_obvalue();
            network_traffic_cost += network_traffic;
            if max_variance < variance{
                max_variance = variance;
            }
        }
    }
    println!("network_traffic: {}, variance:{}, obvalue: {}",network_traffic_cost,max_variance,ALPHA*network_traffic_cost + BETA*max_variance);

}

fn write_topology_json(routers: Vec<Router>) -> std::io::Result<()>{

    //let filename = format!("topology_cluster_{}_with_seed_{}_generated_ratio_oriented.json",k,rand.to_string());
    //let mut file = File::create(filename)?;
    let mut file = File::create("topology.json")?;

    writeln!(file, "{{")?;
    let last_index = routers.len().saturating_sub(1);
    for (i, r) in routers.iter().enumerate() {
        r.print_inner_cluster_devices(&mut file, routers.len())?;
        if i != last_index {
            writeln!(file, ",")?;
        } else {
            writeln!(file)?;
        }
    }
    writeln!(file, "}}")?;

    Ok(())

}

fn clustering(k: usize, d:&Vec<NodeIndex>, time_exec:&DVector<f64>,  routers: &mut Vec<Router>){

    let max_iter = 100;

    let mut device_list = Vec::<Device>::new();
    for i in 0..d.len(){
        let id = d[i];
        let learning_time = time_exec[i];
        let device = Device::new(id, learning_time);
    //    print!("{}, ", learning_time);

        device_list.push(device);
    
    }
    println!("");

    let clustering_result = clustering_device(&device_list, k, max_iter);

    let clustered_group = generate_hashmap(time_exec.as_slice().to_vec(), &d, clustering_result.membership);
    let variance_vec = calculate_variances_clusters(&clustered_group);

    let max_cluster_variance = variance_vec.iter().max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Less)).unwrap();
    allocate_cluster_to_optimal_router(routers, clustered_group, *max_cluster_variance);

}
