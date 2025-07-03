use std::collections::{HashMap};

use petgraph::graph::NodeIndex;

use crate::{sell_info::{SellInfo}};
use crate::router::Router;

pub fn optimize_obvalue_in_cluster(routers: &mut  Vec<Router>) {

/*
    println!("Router list");
for r in routers {
    println!("Router : {:?}", r.get_router());
    }
*/    
//    println!("dist, source, No.device");
    //各ルータのobvalueを計算
    for i in 0..routers.len(){
//            println!("Updating obvalue of router {}",i);
        
        let (_, variance) = routers[i].update_router_value();
        println!("router {} variance: {}",i,variance);
//        println!("router {} NodeIndex: {}",i,routers[i].get_router().index());
    }

//    for s in sell_info_list.iter(){
//       println!("{}", s.view_sell_info());
//    }
    
    let mut routers_nodeindex_hashmap = HashMap::<NodeIndex,usize >::new();
    for (i,r) in routers.iter_mut().enumerate(){
        routers_nodeindex_hashmap.insert(r.get_router(), i);
    }
    let mut variance_hashmap = HashMap::<NodeIndex,f64>::new();
    for i in 0..routers.len(){
    //            println!("Updating obvalue of router {}",i);
            let (_, variance )  = routers[i].update_router_value();
            variance_hashmap.insert(routers[i].get_router(), variance);
            
 //           println!("router {} value: {}",i,routers[i].get_router_value());
        }
        for r in routers.iter_mut()  {
            r.set_variance_vec(variance_hashmap.clone());
        }
        let mut terminate = false;

    loop{
        
        let mut variance_hashmap = HashMap::<NodeIndex,f64>::new();
        
        for i in 0..routers.len(){
            //    println!("Updating obvalue of router {}",i);
            let (_ , variance )  = routers[i].update_router_value();
            variance_hashmap.insert(routers[i].get_router(), variance);
            
        }
        
        for r in routers.iter_mut()  {
            r.set_variance_vec(variance_hashmap.clone());
        }

        for i in 0..routers.len(){
            //    println!("Updating obvalue of router {}",i);
            let (_, _ )  = routers[i].update_router_value();
            //println!("router {}, network_cost: {}, variance: {}",i,network_cost,variance);
            //println!("router value: {}",routers[i].get_router_value());
            
        }
        if terminate{
            break;
        }  


        let mut sell_info_list = Vec::<SellInfo>::new();
        for r in routers.iter_mut(){
            if r.get_devices().len() < 1 
//                || r.get_optimal()
                {
                continue;
            }
            let sell_info_from_router: Vec<SellInfo> = r.sell_all_device();
            sell_info_list.extend(sell_info_from_router);

        }

        // `get_value()` の値が大きい順にソート
//        routers.sort_by(|a, b| b.get_router_value().partial_cmp(&a.get_router_value()).unwrap());


        let mut max_router_value_device_hashmap = HashMap::<NodeIndex, (NodeIndex,f64)>::new();
        let mut remove_router_value_device_hashmap = HashMap::<NodeIndex, Vec<(NodeIndex,f64)>>::new();
        let mut max_value = 0.0;
        let mut max_candidate: (NodeIndex,f64) = (NodeIndex::new(0), 0.0);
        let mut max_value_router: NodeIndex = NodeIndex::new(0);
        
        //To detect loop
//        let mut previous_variance = 0.0;
//        let mut previous_device: NodeIndex<u32>= NodeIndex::new(0);
//        let mut fixed_device = Vec::<NodeIndex>::new();

        for r in routers.iter_mut(){

            if r.get_devices().len() < 1 //|| r.get_is_leaf()
            {
                r.set_optimal(true);
                continue;
            }
            let mut router_updated = false;
//            println!("For router {}",r.get_router().index());
            let mut max_gain_device_value = 0.0;
            for si in sell_info_list.iter() {
                if si.get_router() == r.get_router(){
                    continue;
                }

                let (assume_router_value, max_variance_from_ohters) = r.assume_add_device(si.get_device(), *variance_hashmap.get(&si.get_router()).unwrap(),si.get_variance()); 

                let buyer_improve_value = assume_router_value - r.get_router_value();
                
                let seller_improve_value = if max_variance_from_ohters < si.get_variance(){
                    si.get_after_remove_cost() - si.get_previous_obvalue() 
                }else{
                    //println!("network_cost: {}",si.get_network_cost());
                    //println!("variance: {}",max_variance_from_ohters);
                    //println!("previous_value: {}",si.get_previous_obvalue());
                    //println!("after : {}",crate::ALPHA * si.get_network_cost() - crate::BETA * max_variance_from_ohters );
                    crate::ALPHA * si.get_network_cost() - crate::BETA * max_variance_from_ohters - si.get_previous_obvalue()
                };

               //println!("device name: {:?}, sell_router :{:?}, buy_router: {:?}", si.get_device().0, si.get_router(),r.get_router());
//               println!("seller_previous_value: {}, seller_after_value: {}", si.get_previous_obvalue(), si.get_after_remove_cost());
               //println!("buyer_improve_value: {}, seller_improve_value: {}",buyer_improve_value,seller_improve_value);

               let entire_improvement = seller_improve_value + buyer_improve_value;

//               if fixed_device.contains(&si.get_device().0){
//                    continue;
//               }
                
                if entire_improvement >= 0.0 {
//                println!("assume_router_value:{}, r.get_router_value: {}", assume_router_value,r.get_router_value());
                    let new_candidate = si.get_device();
//                    let max_improve_value = seller_improve_value;
                    if max_gain_device_value < entire_improvement{
                        max_gain_device_value = buyer_improve_value + seller_improve_value;
                        
                        if max_gain_device_value > max_value {
//                            if previous_device == new_candidate.0{
//                                if previous_variance <= si.get_variance(){
//                                    fixed_device.push(new_candidate.0);
//                                    continue;
//                                }
//                            }
                            
                            max_value = max_gain_device_value;
                            max_candidate = new_candidate;
//                            previous_device = new_candidate.0;
//                            previous_variance = si.get_variance();
                            max_value_router = r.get_router();
                        }

                        //println!("device name: {:?}, sell_router :{:?}, buy_router: {:?}", new_candidate, si.get_router(),r.get_router());
                        //println!("buyer_improve_value: {}, seller_improve_value: {}",buyer_improve_value,seller_improve_value);
//                        println!("max_gain_device_value: {}, seller_improve_value: {}", max_gain_device_value, seller_improve_value);
                        router_updated = true;
//                        max_router_value_device_hashmap.entry(r.get_router()).insert_entry(new_candidate);
                    }
                }
            }

            if router_updated == false {
//                println!("set true at router {:?}",r.get_router());
                r.set_optimal(true);
            }
//            else {break;}
        }
        if max_value > 0.0{
            max_router_value_device_hashmap.entry(max_value_router).insert_entry(max_candidate);
        }
        for (_, device) in max_router_value_device_hashmap.iter(){
            for si in sell_info_list.iter(){
                if si.get_device() == *device{
                    remove_router_value_device_hashmap.entry(si.get_router()).or_insert(vec![]).push(*device);
                    break;
                }
            }
        }

        let mut dist: usize = 0;
        let mut dist_router_index: usize = 0;
        let mut source: usize = 0;
        let mut source_router_index:usize = 0;
        let mut device_no: usize = 0;
        let mut moved_device: (NodeIndex,f64) = (NodeIndex::new(0), 0.0);

        let routers_index_map: HashMap<NodeIndex, usize> = routers
                                                        .iter()
                                                        .enumerate()
                                                        .map(|(i, r)| (r.get_router(), i))
                                                        .collect();
   

        // HashMapを使ってVecを更新
        for (nodeindex, device) in max_router_value_device_hashmap.iter() {
            if let Some(&index) = routers_index_map.get(nodeindex) {
                dist = nodeindex.index();
                device_no = device.0.index();
                //routers[index].add_device(*device);
                dist_router_index = index;
            }
        }


        for (nodeindex, devices) in remove_router_value_device_hashmap.iter() {
            if let Some(&index) = routers_index_map.get(nodeindex) {
                for device in devices {
                    source = nodeindex.index();
                    //routers[index].remove_device(*device);
                    source_router_index = index;
                    moved_device = *device;
                }
            }
        }

        if let Some(lca) = find_lca(dist, source) {
            if lca.eq(&source) || lca.eq(&dist) {
                routers[source_router_index].remove_device(moved_device);
                routers[dist_router_index].add_device(moved_device);
            }else {
                println!("Before Detected: {}, {}, {}", dist, source, device_no);
                println!("Redunduncy Detected!! Device:{} will be allocated to Router: {}",moved_device.0.index(),lca);
                routers[source_router_index].remove_device(moved_device);
                routers[lca].add_device(moved_device);
                dist = lca;
            }
        } else {
            routers[source_router_index].remove_device(moved_device);
            routers[dist_router_index].add_device(moved_device);
        }

        println!("{}, {}, {}", dist, source, device_no);
        

//        let mut terminate = false;
        for r in routers.iter_mut(){
            if r.get_optimal() == false{
                terminate = false;
                break;
            } 
            terminate = true;
        }


    }


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
    let original_level = level;

    let mut parent: Option<usize> = None;
    let mut current_id = node_id;
    let mut current_level = level;

    if current_level > 0 {
        let (parent_start, _) = levels[current_level - 1];
        let parent_id_in_prev_level = (current_id - levels[current_level].0) / 4;
        parent = Some(parent_start + parent_id_in_prev_level);
    }

    // top_routerの計算
    while current_level > 0 {
        let (parent_start, _) = levels[current_level - 1];
        let parent_id_in_prev_level = (current_id - levels[current_level].0) / 4;
        let parent_id = parent_start + parent_id_in_prev_level;
        current_id = parent_id;
        current_level -= 1;
    }

    let top_router = current_id;

    Some((original_level, pos_in_level, parent, top_router))
}

/// ノードからrootまでの親チェインをVecで返す
fn get_parent_chain(mut node_id: usize) -> Vec<usize> {
    let mut chain = Vec::new();
    chain.push(node_id);

    while let Some((_level, _pos, parent, _top)) = get_node_info(node_id) {
        if let Some(p) = parent {
            chain.push(p);
            node_id = p;
        } else {
            break;
        }
    }
    chain
}

/// 2ノードの最短共通親を求める
fn find_lca(node1: usize, node2: usize) -> Option<usize> {
    let chain1 = get_parent_chain(node1);
    let chain2 = get_parent_chain(node2);

    for a in &chain1 {
        if chain2.contains(a) {
            return Some(*a);
        }
    }
    None
}
