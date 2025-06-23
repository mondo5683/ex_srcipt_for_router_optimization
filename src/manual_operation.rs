use std::fs::File;
use std::io::Write;

use crate::{algorithms::optimize_obvalue_in_cluster, device, router::{self, Router}};
use petgraph::{graph::NodeIndex};

use crate::{ALPHA,BETA};

pub fn manually_operation(routers: &mut Vec<Router>, router_vec: &Vec<NodeIndex> , device_vec: &Vec<NodeIndex>, time_exec:Vec<f64>) 
-> Result<(), Box<dyn std::error::Error>>{

    println!("Starting Intaractive Mode...");
    loop{
        println!("Please enter commands: ");
        let mut input = "".to_string();
        std::io::stdin().read_line(&mut input).ok();

        let args: Vec<String> = input.split(' ').map(|string| string.replace("\n", "").to_string()).collect();

        match args.len() {
            1 => {
                let cmd = &args[0];
                match &cmd[..] {
                    "status" => {
                        println!("スタッツを表示します。");
                        print_status(routers);
                    },
                    "obvalue" => {
                        println!("obvalueをアップデートします。");
                        calc_obvalue(routers);
                    }
                    "optimize" => {
                        println!("最適化を実行します...");
                        let mut inner_cluster_routers = Vec::<&mut Router>::new();
                        for r in routers.iter_mut() {
                            inner_cluster_routers.push(r);
                        }
                        optimize_obvalue_in_cluster(&mut inner_cluster_routers);
                    },
                    "exit" => {
                        println!("操作を終了します");
                        break()
                    }
                    _ => {
                        eprintln!("Error: invalid command");
                        print_usage();
                    }
                }
            }
            2 => {
                let cmd = &args[0];
                
                match &cmd[..] {
                    "show" => {
                        match parse_nodeindex(&args[1][..]) {
                            Ok(nodeindex) => show(nodeindex, routers, device_vec, &time_exec),
                            Err(msg) => {
                                eprintln!("Systax Error: {}",msg);
                                print_usage();
                            }
                        };
                    },
                    "write" => {
                        let file_name = args[1].to_string();
                        match write_topology(file_name, routers){
                            Ok(_) => println!("Write topology Successfully"),
                            Err(msg) => eprintln!("Error: cannot create file"),
                        }
                    }
                    _ => {
                        eprintln!("Error: invalid command");
                        print_usage();
                    }
                }
            }
            4 => {
                let cmd = &args[0];
                let device_index        = parse_nodeindex(&args[1][..])?; 
                let source_router_index = parse_nodeindex(&args[2][..])?;
                let target_router_index = parse_nodeindex(&args[3][..])?; 

                match &cmd[..] {
                    "mv" => {
                        println!("Move Device: {:?}, Source Node: {:?}, Target Node: {:?}",device_index, source_router_index, target_router_index);
                        let mut is_vaild_args = 0;
                        let router_num = routers.len();
                        let device_num = device_vec.len();

                        if !(router_num..(router_num + device_num)).contains(&device_index){
                            eprintln!("Device Node Indexが不正です。");
                            is_vaild_args+= 1;
                        }
                        if !(0..router_num).contains(&source_router_index) {
                            eprintln!("Source Node Indexが不正です。");
                            is_vaild_args+= 1;
                        }
                        if !(0..router_num).contains(&target_router_index) {
                            eprintln!("Target Node Indexが不正です。");
                            is_vaild_args+= 1;
                        }

                        if is_vaild_args == 0{ 
                            let device_vec_index = device_index - router_num;
                            let device = (device_vec[device_vec_index], time_exec[device_vec_index]);
                            let mut inner_cluster_routers = Vec::<&mut Router>::new();
                            for r in routers.iter_mut() {
                                inner_cluster_routers.push(r);
                            }
                            match move_device(device, source_router_index, target_router_index,&mut inner_cluster_routers) {
                                Ok(_) => println!("Moved Device {:?} from Router {} to Router {}",device,source_router_index,target_router_index),
                                Err(msg) => eprintln!("Error: {}", msg), 
                            }
                        }
                    }
                    _ => {
                        print_usage();
                    }
                }
            }
            
            _ => print_usage(),

        }
    }
    Ok(())

}

fn calc_obvalue(routers: &mut Vec<Router>){
    
    let mut network_traffic_cost = 0.0;
    let mut max_variance = 0.0;
    for r in routers.iter_mut(){
        if r.get_devices().len() > 0{
//                println!("router:{}",r.get_router().index());
            //println!("router:{:?}",r.get_devices());
            let (network_traffic, variance) = r.update_obvalue();
            network_traffic_cost += network_traffic;
            if max_variance < variance{
                max_variance = variance;
            }
        }
    }
    println!("network_traffic: {}, variance:{}, obvalue: {}",network_traffic_cost,max_variance,ALPHA*network_traffic_cost + BETA*max_variance);

}

fn print_status(routers: &Vec<Router>){

    for router in routers.iter(){
        let devices_in_router = router.get_devices();
        
        if devices_in_router.len() == 0 {continue;}

        let nodeindex = router.get_router().index();
        if let Some(results) = get_node_info(nodeindex) {
            let (level, pos, parent, top) = results;
            
            match parent {
                Some(p) => println!("Router: {}, {}階層, {}番目, 親ノード:{}, 最上位ルータ:{}",
                                    nodeindex, level, pos, p, top),
                None => println!("Router: {}, {}階層, {}番目, 親ノードなし, 最上位ルータ:{}",
                                    nodeindex, level, pos, top),

            }
        }
        for (d, time) in devices_in_router {
            println!("--Device: {:?}, time:{}",d,time);
        }
    }


}
fn show(nodeindex:usize, routers: &Vec<Router>, device_vec: &Vec<NodeIndex>, time_exec:&Vec<f64>){

    let router_num = routers.len();
    let device_num = device_vec.len();

    //まずNodeIndexがルータかデバイスかを判別
    if  nodeindex < router_num {
        //Routerの場合，router.get_devices()を呼び出して表示
        if let Some(results) = get_node_info(nodeindex) {
            let (level, pos, parent, top) = results;
            
            match parent {
                Some(p) => println!("Show Router: {}, {}階層, {}番目, 親ノード:{}, 最上位ルータ:{}",
                                    nodeindex, level, pos, p, top),
                None => println!("Show Router: {}, {}階層, {}番目, 親ノードなし, 最上位ルータ:{}",
                                    nodeindex, level, pos, top),

            }
        }
        let devices_in_router = routers[nodeindex].get_devices();

        if devices_in_router.len() == 0 {
            println!("This router has no device");
        }

        for (index, time) in devices_in_router{
            println!("Device: {:?}, time: {}",index,time);
        }
    }else if nodeindex < device_num + router_num {
        //Deviceの場合，deviceの情報を表示
        let device_index = nodeindex - router_num;
        println!("Show Device: {}", nodeindex);
        println!("Device: {:?}, time: {}", device_vec[device_index], time_exec[device_index]);
    }else {
        eprintln!("Error: 引数が不正です。");
    }

}
fn parse_nodeindex(arg: &str) -> Result<usize, String> {
    arg.parse::<usize>().map_err(|_| format!("Error: 引数 '{}' が NodeIndex ではありません。", arg))
}
fn move_device(device: (NodeIndex,f64), source_router_index:usize, target_router_index:usize, routers: &mut Vec<&mut Router>) -> Result<(), String>{

   let device_node_index = device.0;
   let is_in_source_router = routers[source_router_index].get_devices().iter().any(|(index, _)| *index == device_node_index);

   if is_in_source_router {
        routers[source_router_index].remove_device(device);
        routers[target_router_index].add_device(device);
        Ok(())
   }else {
        Err(format!("Device {:?} はRouter: {}に所有されていません。",device,source_router_index))
   }
   
    

}
fn print_usage(){

    println!("Usage:");
    println!("  status                     - Show current status");
    println!("  obvalue                    - Update OB values");
    println!("  optimize                   - Run optimization");
    println!("  exit                       - Exit the program");
    println!("  write  <file_name>         - write topology as file <file_name>");
    println!("  show <node_index>          - Show information of the node");
    println!("  mv <dev_idx> <src> <tgt>   - Move device from source router to target router");


}

fn write_topology(file_name: String, routers: &Vec<Router>) -> std::io::Result<()> {

    let mut file = File::create(file_name)?;
    
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

fn get_node_info(node_id: usize) -> Option<(usize, usize, Option<usize>, usize)> {
    if node_id >= 1190 {
        return None;
    }

    let levels = [
        (0, 14),     // level 0
        (14, 56),    // level 1
        (70, 224),   // level 2
        (294, 896),  // level 3
    ];

    // レベルと位置を判定
    let mut level = 0;
    let mut start = 0;
    for (i, &(s, count)) in levels.iter().enumerate() {
        if node_id >= s && node_id < s + count {
            level = i;
            start = s;
            break;
        }
    }

    let pos_in_level = node_id - start;
    let original_level = level; // ← ここでコピー

    // 親とトップルータを探す処理
    let mut parent: Option<usize> = None;
    let mut current_id = node_id;
    while level > 0 {
        let (parent_start, _) = levels[level - 1];
        let parent_id_in_prev_level = (current_id - levels[level].0) / 4;
        let parent_id = parent_start + parent_id_in_prev_level;
        parent = Some(parent_id);
        current_id = parent_id;
        level -= 1;
    }

    let top_router = current_id;

    Some((original_level, pos_in_level, parent, top_router))
}



