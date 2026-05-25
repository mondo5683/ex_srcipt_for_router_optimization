use graph_test::*;

//For 最短経路 TEST用
use petgraph::{algo, prelude::*};

fn main() -> std::io::Result<()> {
    let rand_seeds: [u64; 10] = [100, 101, 102, 103, 104, 105, 106, 107, 108, 109];
    //let rand_seeds: [u64; 1] = [101];
    //    let k_vec = [1,2,3,4,5,6,7,8,9,10,11,12,13];
    //    let k_vec = [9,10,11,12,13];
    //    let k_vec = [1,2,3,4];
    let model_size = 500.0;

    let ps_allocated_router: [usize; 10] = [524, 281, 1090, 1178, 1183, 224, 1108, 832, 280, 922];

    for (_i, &rand) in rand_seeds.iter().enumerate() {
        println!("Seed: {}", rand);
        //for k in k_vec{ println!("For k: {}", k);

        let (_d, _time_exec, mut routers) = prepare_routers(rand, model_size);

        print_current_obvalue(&mut routers);
        /*
        match manually_operation(&mut routers, &r, &d, time_exec.column(0).iter().map(|x| *x).collect()){
            Ok(_) => println!("Manual Operation completed."),
            Err(msg) => eprintln!("Manual Operation stops due to error {}",msg),

        };
        */
        let _k = 75;
        clustering(_k, &_d, &_time_exec, &mut routers);
        write_topology_json(&routers)?;

        // Router Hop Count
        /*
        let ps_router = ps_allocated_router[_i];
        let mut most_hopped_router_count: HashMap<NodeIndex, usize> = HashMap::new();

        for d in _d {
            let r = routers[ps_router].get_router();
            let gr = routers[0].get_topology();
            let res = algo::astar(&**gr, d, |n| n == r, |e| 1, |_| 0);

            if let Some((_hop, path)) = res {
                for r in path {
                    if !(r == d) {
                        *most_hopped_router_count.entry(r).or_insert(0) += 1;
                    }
                }
                //println!("Distance: {}, Path: {:?}", hop, path);
            }
        }
        let mut vec: Vec<(&NodeIndex, &usize)> = most_hopped_router_count.iter().collect();
        vec.sort_by(|a, b| b.1.cmp(&a.1));
        println!("sort result: {:?}", vec);
        */ //-- END Router Hop Count

        // let is_first_round = true;
        // println!("Starting Initial optimization...");
        // for r in routers.iter_mut(){
        //     r.set_alpha(0.001);
        //     r.set_beta(0.999);
        //     r.set_is_first_round(true);
        // }
        // optimize_obvalue_in_cluster(&mut routers, is_first_round);
        print_current_obvalue(&mut routers);
        // println!("Starting Second optimization...");
        // for r in routers.iter_mut(){
        //     r.set_alpha(0.5);
        //     r.set_beta(0.5);
        //     r.set_optimal(false);
        //     r.set_is_first_round(false);
        // }
        let is_first_round = false;
        optimize_obvalue_in_cluster(&mut routers, is_first_round);
        print_current_obvalue(&mut routers);
        println!("---");

        //    println!("router num:{}",router_num);
        //write_topology_json(routers)?;
        //The followed } is the end of loop of k
        //}
    }

    Ok(())
}
