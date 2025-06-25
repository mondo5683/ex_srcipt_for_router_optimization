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

use std::collections::HashMap;

use gen_topology::{generate_device_array, generate_edge_array, generate_router_array};
use na::{DMatrix, DVector};
use petgraph::{algo::all_simple_paths, Graph, Undirected};
use random_generation::{generate_random_edge_distribution, generate_random_time_exec, random_locate_ps};
use router::Router;
use algorithms::optimize_obvalue_in_cluster;
use rand::{rngs::StdRng, Rng, SeedableRng};
use device::Device;
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

    for rand in rand_seeds{
        println!("Seed: {}", rand);
        for k in k_vec{ println!("For k: {}", k);
    //    let time_exec = na::vector![20.0,100.0,35.0,100.0,60.0,300.0,40.0];
    //    let time_exec = na::vector![20.0,35.0,100.0,100.0,80.0,300.0,40.0];
        let model_size = 500.0;

        let mut rng = StdRng::seed_from_u64(rand);
        let te_seed:u64 = rng.random();
        let ed_seed:u64 = rng.random();
        let ps_location_seed:u64 = rng.random();

        //let time_exec = generate_random_time_exec(DEVICE_NUM, te_seed, TE_MEAN, TE_STD);
        let server_rate = 0.1;
        let desktop_rate= 0.3;
        let laptop_rate = 0.3;
        let raspi_rate  = 0.2;
        let sensor_rate = 0.1;

        let hashmap_device_type_to_ratio = HashMap::from([
            ("server", server_rate),
            ("desktop", desktop_rate),
            ("laptop", laptop_rate),
            ("raspi", raspi_rate),
            ("sensor", sensor_rate),
        ]);

        let mut hashmap_timeexec_kind_to_mean_and_std: HashMap<&str,(f64,f64)> = HashMap::new();
        hashmap_timeexec_kind_to_mean_and_std.insert("server", (15.0,3.0));
        hashmap_timeexec_kind_to_mean_and_std.insert("desktop", (40.0,8.0));
        hashmap_timeexec_kind_to_mean_and_std.insert("laptop", (80.0,25.0));
        hashmap_timeexec_kind_to_mean_and_std.insert("raspi", (150.0, 15.0));
        hashmap_timeexec_kind_to_mean_and_std.insert("sensor", (400.0, 80.0));

        let mut time_exec_buff = Vec::<f64>::new();
        for (device_type, mean_and_std) in hashmap_timeexec_kind_to_mean_and_std{
            let num_samples:usize = ( DEVICE_NUM as f64 * *hashmap_device_type_to_ratio.get(device_type).unwrap() ) as usize;
            let time_exec_for_the_device_type = generate_random_time_exec(num_samples, te_seed, mean_and_std.0, mean_and_std.1);

            let time_exec_for_the_device_type_vec = time_exec_for_the_device_type.iter().copied().collect::<Vec<f64>>();
            time_exec_buff.extend(time_exec_for_the_device_type_vec);

        }

        let time_exec = DVector::from_vec(time_exec_buff.clone());
        let e = generate_random_edge_distribution(rt.nrows(), DEVICE_NUM, ed_seed);
    //    println!("{:?}", e);
        println!("{:?}",time_exec);

        let mut topology = Graph::<String, String, Undirected>::new_undirected();
        
        let r = generate_router_array(&rt, &mut topology);
        let d = generate_device_array(&e, &mut topology);

        let mut device_list = Vec::<Device>::new();
        for i in 0..d.len(){
            let id = d[i];
            let learning_time = time_exec[i];
            let device = Device::new(id, learning_time);
        //    print!("{}, ", learning_time);

            device_list.push(device);
        
        }
        println!("");

        // Create Vertexes
        let ps = topology.add_node(String::from("parameter_server"));
        let mut edge_array = generate_edge_array(&rt, &e, &r, &d);
        edge_array.push(random_locate_ps(rt.nrows(), &ps, &r, ps_location_seed));
        //edge_array.push((ps, r[143]));
        topology.extend_with_edges(&edge_array);


    //    println!("{:?}",topology);
        let topology_for_routers = topology.clone();


    //        let time_exec_average = time_exec.iter().sum::<f64>() as f64 / time_exec.len() as f64 ;

    //        println!("Time execution average: {}", time_exec_average);

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
      

        //TODO delete this
        //let device9 = (d[9],time_exec[9]); 
        //routers[1].add_device(device9);
        //routers[2].remove_device(device9);
       

        let mut network_traffic_cost = 0.0;
        let mut max_variance = 0.0;
        for r in routers.iter_mut(){
            if r.get_devices().len() > 0{
//                println!("router:{}",r.get_router().index());
                println!("router:{:?}",r.get_devices());
            
                let (network_traffic, variance) = r.update_obvalue();
                network_traffic_cost += network_traffic;
                if max_variance < variance{
                    max_variance = variance;
                }
            }
        }
        println!("network_traffic: {}, variance:{}, obvalue: {}",network_traffic_cost,max_variance,ALPHA*network_traffic_cost + BETA*max_variance);
    //    println!("router num:{}",router_num);
        
        /*
        let mut routers_stratum_dict:HashMap<usize,Vec<usize>> = HashMap::new();
        let mut max_depth = 0;
        for r in routers.iter() {
            let depth_of_r = r.hop_to_node(&ps);
    //        println!("router {:?} is {}",r.get_router(), depth_of_r));
            routers_stratum_dict.entry(depth_of_r).or_insert(vec![]).push(r.get_router().index());

            if max_depth < depth_of_r {
                max_depth = depth_of_r
            }
        }
        */
        
        /*
        match manually_operation(&mut routers, &r, &d, time_exec_buff){
            Ok(_) => println!("Manual Operation completed."),
            Err(msg) => eprintln!("Manual Operation stops due to error {}",msg),

        };
        */
        
        
        

        

    /*    println!("Max depth : {}", max_depth);
        
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
    */

        let mut inner_cluster_routers = Vec::<&mut Router>::new();
        for r in routers.iter_mut() {
            inner_cluster_routers.push(r);
        }

        //let k = 8;
        let max_iter = 100;
        let clustering_result = clustering_device(&device_list, k, max_iter);

        let clustered_group = generate_hashmap(time_exec.as_slice().to_vec(), &d, clustering_result.membership);
        let variance_vec = calculate_variances_clusters(&clustered_group);

        let max_cluster_variance = variance_vec.iter().max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Less)).unwrap();
        allocate_cluster_to_optimal_router(&mut inner_cluster_routers, clustered_group, *max_cluster_variance);
        //optimize_obvalue_in_cluster(&mut inner_cluster_routers);

        network_traffic_cost = 0.0;
        max_variance = 0.0;
        
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
        println!("---");
//    println!("router num:{}",router_num);

        let filename = format!("topology_cluster_{}_with_seed_{}_generated_ratio_oriented.json",k,rand.to_string());
        let mut file = File::create(filename)?;
        //let mut file = File::create("topology.json")?;
    
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

//この関数によって生成されるrouterのlifetimeはトポロジーの配列rtと同値になります
/*
fn generete_router_vec(rand:u64, model_size: f64, rt:&na::DMatrix<i32>) -> Vec<Router> {

    let mut rng = StdRng::seed_from_u64(rand);
    let te_seed:u64 = rng.random();
    let ed_seed:u64 = rng.random();
    let ps_location_seed:u64 = rng.random();

    //let time_exec = generate_random_time_exec(DEVICE_NUM, te_seed, TE_MEAN, TE_STD);
    let server_rate = 0.1;
    let desktop_rate= 0.3;
    let laptop_rate = 0.3;
    let raspi_rate  = 0.2;
    let sensor_rate = 0.1;

    let hashmap_device_type_to_ratio = HashMap::from([
        ("server", server_rate),
        ("desktop", desktop_rate),
        ("laptop", laptop_rate),
        ("raspi", raspi_rate),
        ("sensor", sensor_rate),
    ]);

    let mut hashmap_timeexec_kind_to_mean_and_std: HashMap<&str,(f64,f64)> = HashMap::new();
    hashmap_timeexec_kind_to_mean_and_std.insert("server", (15.0,3.0));
    hashmap_timeexec_kind_to_mean_and_std.insert("desktop", (40.0,8.0));
    hashmap_timeexec_kind_to_mean_and_std.insert("laptop", (80.0,25.0));
    hashmap_timeexec_kind_to_mean_and_std.insert("raspi", (150.0, 15.0));
    hashmap_timeexec_kind_to_mean_and_std.insert("sensor", (400.0, 80.0));

    let mut time_exec_buff = Vec::<f64>::new();
    for (device_type, mean_and_std) in hashmap_timeexec_kind_to_mean_and_std{
        let num_samples:usize = ( DEVICE_NUM as f64 * *hashmap_device_type_to_ratio.get(device_type).unwrap() ) as usize;
        let time_exec_for_the_device_type = generate_random_time_exec(num_samples, te_seed, mean_and_std.0, mean_and_std.1);

        let time_exec_for_the_device_type_vec = time_exec_for_the_device_type.iter().copied().collect::<Vec<f64>>();
        time_exec_buff.extend(time_exec_for_the_device_type_vec);

    }

    let time_exec = DVector::from_vec(time_exec_buff.clone());
    let e = generate_random_edge_distribution(rt.nrows(), DEVICE_NUM, ed_seed);
//    println!("{:?}", e);
    println!("{:?}",time_exec);

    let mut topology = Graph::<String, String, Undirected>::new_undirected();
    
    let r = generate_router_array(&rt, &mut topology);
    let d = generate_device_array(&e, &mut topology);

    let mut device_list = Vec::<Device>::new();
    for i in 0..d.len(){
        let id = d[i];
        let learning_time = time_exec[i];
        let device = Device::new(id, learning_time);
    //    print!("{}, ", learning_time);

        device_list.push(device);
    
    }
    println!("");

    // Create Vertexes
    let ps = topology.add_node(String::from("parameter_server"));
    let mut edge_array = generate_edge_array(&rt, &e, &r, &d);
    //edge_array.push(random_locate_ps(rt.nrows(), &ps, &r, ps_location_seed));
    edge_array.push((ps, r[105]));
    topology.extend_with_edges(&edge_array);


//    println!("{:?}",topology);
    let topology_for_routers = topology.clone();


//        let time_exec_average = time_exec.iter().sum::<f64>() as f64 / time_exec.len() as f64 ;

//        println!("Time execution average: {}", time_exec_average);

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
    routers

}
*/
