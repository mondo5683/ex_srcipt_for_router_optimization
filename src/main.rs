use graph_test::*;

fn main() -> std::io::Result<()> {

    let rand_seeds: [u64; 10] = [100,101,102,103,104,105,106,107,108,109];
//    let rand_seeds: [u64; 1] = [100];
//    let k_vec = [1,2,3,4,5,6,7,8,9,10,11,12,13];
//    let k_vec = [9,10,11,12,13];
    let k_vec = [1,2,3,4];
    let model_size = 500.0;


    for rand in rand_seeds{
        println!("Seed: {}", rand);
        //for k in k_vec{ println!("For k: {}", k);

        let (d, time_exec,mut routers) = prepare_routers(rand, model_size);

        print_current_obvalue(&mut routers);
        /*
        match manually_operation(&mut routers, &r, &d, time_exec.column(0).iter().map(|x| *x).collect()){
            Ok(_) => println!("Manual Operation completed."),
            Err(msg) => eprintln!("Manual Operation stops due to error {}",msg),

        };
        */
        let k = 3;
//        clustering(k, &d, &time_exec ,&mut routers);
        optimize_obvalue_in_cluster(&mut routers);
        print_current_obvalue(&mut routers);
        println!("---");

//    println!("router num:{}",router_num);
//      write_topology_json(routers)?;
    //The followed } is the end of loop of k
    //}
    }

    Ok(())
}


