extern crate petgraph;
extern crate nalgebra as na;
mod router;
mod graph_operation;
mod sell_info;
mod gen_topology;
mod random_generation;

use gen_topology::{generate_device_array, generate_edge_array, generate_router_array};
use petgraph::{algo::all_simple_paths, graph::NodeIndex, Graph, Undirected};
use router::Router;
use random_generation::{generate_random_edge_distribution, generate_random_time_exec};

const PRINT_CSV_MODE:bool = true;
const MAX_VARIANCE:f64 = 400.0;
const MIN_COMM_COST:f64 = 7500.0;

const ALPHA:f64 = 0.5;
const BETA:f64 = 0.5;
const INF:f64 = 9999999999999999.0;

//Parameters for random_generation
const DEVICE_NUM: usize = 10;
const TE_MEAN : f64 = 100.0;
const TE_STD:f64 = 95.0;
const TE_SEED: u64 = 123;
const ED_SEED: u64 = 123;


fn set_all_pattern(a: &mut na::DMatrix<i32>, column_index: i32, routers: &mut Vec::<Router>, 
                   d:&Vec::<NodeIndex>, time_exec:&Vec::<f64>){
    
    if column_index < 0{

        //vecに応じてデバイスを割り当てる
        let mut device_list_for_router = Vec::<Vec<(NodeIndex,f64)>>::with_capacity(routers.len());
        // 各ルーターの数だけ空のベクターを追加
        for _ in 0..routers.len() {
            device_list_for_router.push(Vec::new());
        }

        for (col_index, column) in a.column_iter().enumerate() {
            for (row_index, value) in column.iter().enumerate() {
                if *value == 1 {
                    let device = (d[col_index], time_exec[col_index]);
                    device_list_for_router[row_index].push(device);
                }
            }
        }
        let mut max_of_variance = 0.0;
        let mut sum_of_comm_cost = 0.0;
        for (i,r) in routers.iter_mut().enumerate(){
            let (comm_cost, variance) = r.calc_obvalue_return_two_variables(&device_list_for_router[i]);
            sum_of_comm_cost += comm_cost;
            
            if max_of_variance < variance {
                max_of_variance = variance;
            }
        }
        if PRINT_CSV_MODE {
            println!("{},{}, {}",sum_of_comm_cost,max_of_variance, sum_of_comm_cost * ALPHA + max_of_variance * BETA);
        }
        else if sum_of_comm_cost == MIN_COMM_COST && max_of_variance < MAX_VARIANCE{
            println!("{},{}",sum_of_comm_cost,max_of_variance);
            println!("{:?}",a);
         }

//        println!("{:?}",A);
        return;
    }

    for i in 1..a.nrows() {
        set_all_pattern(a, column_index - 1,routers,d,time_exec);
        let c = column_index as usize;
        a[(i,c)] = 1;
        a[(i-1,c)] = 0;
    }
    set_all_pattern(a, column_index - 1,routers,d,time_exec);

    //自分の列を初期化
    for i in 0..a.nrows(){
        let c = column_index as usize;
        a[(i,c)] = 
            if i == 0{
                1
            }else {0};
    }
}

fn main() {
/*    let gr : UnGraph<(),()> = Graph::from_edges(&[
        (0,1), (0,2), (0,3), 
        (1,3),
        (2,3),(2,4),
        (4,0),(4,5),]);
*/
    let rt = na::dmatrix![
        0,1,1,1;
        0,0,0,0;
        0,0,0,0;
        0,0,0,0;
    ];

    let time_exec = generate_random_time_exec(DEVICE_NUM, TE_SEED, TE_MEAN, TE_STD);
    let e = generate_random_edge_distribution(rt.nrows(), DEVICE_NUM, ED_SEED);
    println!("{:?}", e);
    println!("{:?}",time_exec);
 
    let mut topology = Graph::<String, String, Undirected>::new_undirected();

    // Create Vertexes
    let ps = topology.add_node(String::from("parameter_server"));
    let r = generate_router_array(&rt, &mut topology);
    let d = generate_device_array(&e, &mut topology);
    
    let model_size = 500.0;
    let topology_for_routers = topology.clone();
    
    let mut routers = Vec::<Router>::new();
    for router in r.iter(){
        routers.push(Router::new(&topology_for_routers, router,&ps, model_size));
    } 
 

    let mut topology = Graph::<String, String, Undirected>::new_undirected();
    

    // Create Vertexes
    let ps = topology.add_node(String::from("parameter_server"));
    let edge_array = generate_edge_array(&rt, &e, &r, &d, &ps);
    topology.extend_with_edges(&edge_array);

    let model_size = 500.0;
    println!("{:?}",topology);
    let topology_for_routers = topology.clone();


    let time_exec_average = time_exec.iter().sum::<f64>() as f64 / time_exec.len() as f64 ;

//    println!("Time execution average: {}", time_exec_average);


    let mut routers = Vec::<Router>::new();
    for router in r.iter(){
        routers.push(Router::new(&topology_for_routers, router,&ps, model_size));
    } 
    

    let mut a = na::dmatrix![
        1,1,1,1,1,1,1,1,1,1;
        0,0,0,0,0,0,0,0,0,0;
        0,0,0,0,0,0,0,0,0,0;
        0,0,0,0,0,0,0,0,0,0;
    ];


//    let time_exec = na::vector![20.0,100.0,35.0,100.0,60.0,300.0,40.0];
//    let time_exec = na::vector![20.0,35.0,100.0,100.0,80.0,300.0,40.0];

    println!("sum_of_comm_cost, max_of_variance, obvalue");
    set_all_pattern(&mut a, DEVICE_NUM as i32 - 1 ,&mut routers,&d,&time_exec.as_slice().to_vec());


   
//    println!("comm cost: {}, variance:{}",sum_of_comm_cost,max_of_variance);

  
}
