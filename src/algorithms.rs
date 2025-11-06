use std::collections::HashMap;

use petgraph::graph::NodeIndex;
use petgraph::Direction::Incoming;

use crate::router::Router;
use crate::sell_info::SellInfo;
use crate::{ALPHA, BETA};
use std::time;

pub fn optimize_obvalue_in_cluster(routers: &mut Vec<Router>) {
    //各ルータのobvalueを計算
    for i in 0..routers.len() {
        let (_, _variance) = routers[i].update_router_value();
        //        println!("router {} variance: {}",i,variance);
    }

    let mut routers_nodeindex_hashmap = HashMap::<NodeIndex, usize>::new();
    for (i, r) in routers.iter_mut().enumerate() {
        routers_nodeindex_hashmap.insert(r.get_router(), i);
    }

    let mut variance_hashmap = HashMap::<NodeIndex, f64>::new();
    update_max_variance_hashmap(routers, &mut variance_hashmap);

    let mut terminate = false;
    let mut _stop_flag = false;

    loop {
        update_max_variance_hashmap(routers, &mut variance_hashmap);
        for i in 0..routers.len() {
            routers[i].update_router_value();
        }
        let (&max_variance_router, _) = variance_hashmap
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .unwrap();
        routers[max_variance_router.index()].set_is_max_variance_router(true);

        if terminate {
            break;
        }
        let sell_info_list = generate_sell_info_list(routers);

        let (max_router_value_device_hashmap, remove_router_value_device_hashmap) =
            search_maximized_swap_node(routers, &sell_info_list, &variance_hashmap);

        swap_device_between_routers(
            routers,
            &max_router_value_device_hashmap,
            &remove_router_value_device_hashmap,
        );

        //        let mut terminate = false;
        for r in routers.iter_mut() {
            if r.get_optimal() == false {
                terminate = false;
                break;
            }
            terminate = true;
        }
    }
}

fn update_max_variance_hashmap(
    routers: &mut Vec<Router>,
    variance_hashmap: &mut HashMap<NodeIndex, f64>,
) {
    for i in 0..routers.len() {
        let (_, variance) = routers[i].update_router_value();
        variance_hashmap.insert(routers[i].get_router(), variance);
    }

    for r in routers.iter_mut() {
        r.set_variance_vec(variance_hashmap.clone());
    }
}

fn generate_sell_info_list(routers: &mut Vec<Router>) -> Vec<SellInfo> {
    let mut sell_info_list = Vec::<SellInfo>::new();
    for r in routers.iter_mut() {
        if (r.get_devices().len() > 0) && (r.get_is_updated()) {
            // || r.has_max_variance_router()) {
            let sell_info_from_router: Vec<SellInfo> = r.sell_all_device();
            sell_info_list.extend(sell_info_from_router);
        }
    }

    sell_info_list
}

fn calc_improvement_using_si_for_router(
    r: &mut Router,
    si: &SellInfo,
    variance_hashmap: &HashMap<NodeIndex, f64>,
) -> (f64, f64, f64) {
    let (assume_router_value, max_variance_from_others, add_assume_variance) = r.assume_add_device(
        si.get_device(),
        *variance_hashmap.get(&si.get_router()).unwrap(),
        si.get_after_remove_variance(),
    );

    let buyer_improve_value = assume_router_value - r.get_router_value();

    //TODO 一応これは政界ではない
    //正確には2番目のvarianceを覚えておく必要があるかもしれない
    let seller_improve_value = if max_variance_from_others < si.get_after_remove_variance() {
        si.get_after_remove_cost() - si.get_previous_obvalue()
    } else {
        ALPHA * si.get_network_cost() - BETA * max_variance_from_others - si.get_previous_obvalue()
    };

    (
        buyer_improve_value,
        seller_improve_value,
        add_assume_variance,
    )
}

fn search_maximized_swap_node(
    routers: &mut Vec<Router>,
    sell_info_list: &Vec<SellInfo>,
    variance_hashmap: &HashMap<NodeIndex, f64>,
) -> (
    HashMap<NodeIndex, (NodeIndex, f64)>,
    HashMap<NodeIndex, Vec<(NodeIndex, f64)>>,
) {
    let mut max_router_value_device_hashmap = HashMap::<NodeIndex, (NodeIndex, f64)>::new();
    let mut remove_router_value_device_hashmap = HashMap::<NodeIndex, Vec<(NodeIndex, f64)>>::new();
    let mut max_value = 0.0;
    let mut max_candidate: (NodeIndex, f64) = (NodeIndex::new(0), 0.0);
    let mut max_value_router: NodeIndex = NodeIndex::new(0);
    let mut max_sell_info: Option<SellInfo> = None;

    let (router, max_variance_from_hasmap) = variance_hashmap
        .iter()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .unwrap();
    println!(
        "max_variance: {}, router_id: {}",
        *max_variance_from_hasmap,
        router.index()
    );

    let mut now: Option<time::Instant> = None;
    let mut is_first_router = true;

    for r in routers.iter_mut() {
        if r.get_devices().len() < 1 {
            r.set_optimal(true);
            continue;
        }

        if is_first_router {
            now = Some(time::Instant::now());
            let first_router_id = r.get_router().index();
            let number_of_devices = r.get_devices().len();
            println!(
                "first_router: {}, #devices: {}",
                first_router_id, number_of_devices
            );
            is_first_router = false;
        }

        let mut results = Vec::new();
        for (index, (_, sell_info)) in r.get_improve_queue().iter_mut().enumerate() {
            //  let before_remove_variance = sell_info.get_before_remove_variance();
            //  if *max_variance_from_hasmap <= before_remove_variance {
            // 必要データを clone
            let sell_info_copy = sell_info.clone(); // Clone が必要
            results.push((index, sell_info_copy));
            // }
        }
        /*
                if is_first_router {
                    if let Some(t) = &now {
                        println!("sell_info copy {:?}", t.elapsed());
                    }
                }
        */
        // improve_queue 内の該当する improve_value を更新
        // 始めに自身の持つsell_info_listに直前のトレードを反映し更新させる
        let mut decreased_variance = 0.0;
        for (index, sell_info_copy) in results.iter_mut() {
            // println!("Update value with {}", improve_value);
            //一回目のtrade roundでないなら更新する必要がある（このifの処理が実行される）
            if let Some(max_variance_before_round) = r.get_max_variance_before_round() {
                let previous_obvalue = sell_info_copy.get_previous_obvalue();
                //各sell_infoのvariance部分を改善した分下げる
                decreased_variance = max_variance_before_round - *max_variance_from_hasmap;
                r.get_improve_queue()[*index]
                    .1
                    .set_previous_obvalue(previous_obvalue + BETA * decreased_variance);
                sell_info_copy.set_previous_obvalue(previous_obvalue + BETA * decreased_variance);

                //updateする対象がmax_varianceを持つ対象の場合そのまま更新
                if *max_variance_from_hasmap == sell_info_copy.get_before_remove_variance() {
                    let network_cost = sell_info_copy.get_network_cost();
                    let after_remove_variance = sell_info_copy.get_after_remove_variance();
                    let up_to_date_after_remove_obvalue =
                        ALPHA * network_cost - BETA * after_remove_variance;
                    r.get_improve_queue()[*index]
                        .1
                        .set_after_remove_obvalue(up_to_date_after_remove_obvalue);
                    sell_info_copy.set_after_remove_obvalue(up_to_date_after_remove_obvalue);
                //そうでない場合はvarianceのみを減少量分下げる
                } else {
                    let after_remove_obvalue = sell_info_copy.get_after_remove_cost();
                    let up_to_date_after_remove_obvalue =
                        after_remove_obvalue + BETA * decreased_variance;
                    r.get_improve_queue()[*index]
                        .1
                        .set_after_remove_obvalue(up_to_date_after_remove_obvalue);
                    sell_info_copy.set_after_remove_obvalue(up_to_date_after_remove_obvalue);
                }
            }
            //            if is_first_router {
            //                if let Some(t) = &now {
            //                    println!("Update improve queue 1 {:?}", t.elapsed());
            //                }
            //            }

            //let (buyer_improve_value, seller_improve_value, add_assume_variance) =
            //    calc_improvement_using_si_for_router(r, &sell_info_copy, variance_hashmap);
            // For acceleralating process, Skip calc_improvement_using_si_for_router() if it's unnessesary
            let before_add_assume_variance =
                r.get_improve_queue()[*index].0.get_assume_add_variance();
            let is_variance_checked = r.get_improve_queue()[*index].0.is_variance_checked();

            let (buyer_improve_value, seller_improve_value, add_assume_variance) = if r
                .get_is_updated()
            {
                r.get_improve_queue()[*index].0.set_variance_checked(false);
                calc_improvement_using_si_for_router(r, &sell_info_copy, variance_hashmap)
            } else if before_add_assume_variance > *max_variance_from_hasmap {
                if !is_variance_checked {
                    r.get_improve_queue()[*index].0.set_variance_checked(true);
                    calc_improvement_using_si_for_router(r, &sell_info_copy, variance_hashmap)
                } else {
                    let before_buyer = r.get_improve_queue()[*index].0.get_buyer_improved_value()
                        - BETA * decreased_variance;
                    let before_seller = r.get_improve_queue()[*index].0.get_seller_improved_value()
                        - BETA * decreased_variance;
                    (before_buyer, before_seller, before_add_assume_variance)
                }
            } else {
                let before_variance = sell_info_copy.get_before_remove_variance();
                let (buyer_improve_value, seller_improve_value) = if before_variance
                    == *max_variance_from_hasmap
                {
                    let (biv, siv, _) =
                        calc_improvement_using_si_for_router(r, &sell_info_copy, variance_hashmap);
                    (biv, siv)
                } else {
                    let before_buyer = r.get_improve_queue()[*index].0.get_buyer_improved_value();
                    let before_seller = r.get_improve_queue()[*index].0.get_seller_improved_value();
                    (before_buyer, before_seller)
                };
                (
                    buyer_improve_value,
                    seller_improve_value,
                    before_add_assume_variance,
                )
            };

            let improve_value = buyer_improve_value + seller_improve_value;
            r.get_improve_queue()[*index]
                .0
                .set_buyer_improved_value(buyer_improve_value);
            r.get_improve_queue()[*index]
                .0
                .set_seller_improved_value(seller_improve_value);
            r.get_improve_queue()[*index]
                .0
                .set_improve_value(improve_value);
            r.get_improve_queue()[*index]
                .0
                .set_assume_add_variance(add_assume_variance);
            //            if is_first_router {
            //                is_first_router = false;
            //                if let Some(t) = &now {
            //                    println!("Update improve queue 2 {:?}", t.elapsed());
            //                }
            //            }
        }

        for si in sell_info_list.iter() {
            if si.get_router() == r.get_router() {
                continue;
            }

            let (buyer_improve_value, seller_improve_value, add_assume_variance) =
                calc_improvement_using_si_for_router(r, si, &variance_hashmap);

            let improvement_for_si = buyer_improve_value + seller_improve_value;
            //    if improvement_for_si > 0.0 {
            r.insert_to_improve_queue(
                improvement_for_si,
                buyer_improve_value,
                seller_improve_value,
                add_assume_variance,
                si.clone(),
            );
            //    }
        }
        //r.get_improve_queue().retain(|&(value ,_ )| value > -600.0 );
        r.set_max_variance_before_round(*max_variance_from_hasmap);
        let _r_index = r.get_router().index();
        match r.get_improve_queue().iter().max_by(|a, b| {
            a.0.get_improve_value()
                .partial_cmp(&b.0.get_improve_value())
                .unwrap()
        }) {
            Some(result) => {
                let (queue_data, sell_info) = result;

                let max_gain_for_this_router = queue_data.get_improve_value();
                if max_value < max_gain_for_this_router {
                    max_value = max_gain_for_this_router;
                    max_candidate = sell_info.get_device();
                    max_sell_info = Some(sell_info.clone());
                    //let source_router_index = sell_info.get_router();
                    //remove_router_value_device_hashmap.entry(source_router_index).or_insert(vec![]).push(max_candidate);
                    //                    println!(" max_gain: {}, device: {:?}, source_router: {}, receive_router: {}", max_gain_for_this_router,
                    //                             sell_info.get_device(), sell_info.get_router().index(), _r_index );
                    max_value_router = r.get_router();
                }
            }
            None => {
                r.set_optimal(true);
            }
        }
    }
    if max_value > 0.0 {
        max_router_value_device_hashmap
            .entry(max_value_router)
            .insert_entry(max_candidate);
    }

    if let Some(si) = max_sell_info {
        let source_router_index = si.get_router();
        remove_router_value_device_hashmap
            .entry(source_router_index)
            .or_insert(vec![])
            .push(max_candidate);
        //          si.print_sell_info();
    };

    (
        max_router_value_device_hashmap,
        remove_router_value_device_hashmap,
    )
}

fn swap_device_between_routers(
    routers: &mut Vec<Router>,
    max_router_value_device_hashmap: &HashMap<NodeIndex, (NodeIndex, f64)>,
    remove_router_value_device_hashmap: &HashMap<NodeIndex, Vec<(NodeIndex, f64)>>,
) {
    let mut dist: usize = 0;
    let mut dist_router_index: usize = 0;
    let mut source: usize = 0;
    let mut source_router_index: usize = 0;
    let mut device_no: usize = 0;
    let mut moved_device: Option<(NodeIndex, f64)> = None;

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
            dist_router_index = index;
        }
    }

    for (nodeindex, devices) in remove_router_value_device_hashmap.iter() {
        if let Some(&index) = routers_index_map.get(nodeindex) {
            //println!("{:?}",devices);
            for device in devices {
                source = nodeindex.index();
                source_router_index = index;
                moved_device = Some(*device);
            }
        }
    }

    if let Some(device) = moved_device {
        //LCAを検出しそこに配置
        if let Some(lca) = find_lca(dist, source) {
            if !(lca.eq(&source) || lca.eq(&dist)) {
                println!(
                    "Redunduncy Detected!! Device:{} will be allocated to Router: {}",
                    device.0.index(),
                    lca
                );
                dist_router_index = lca;
                dist = lca;

                if routers[lca].get_improve_queue().len() < 1 {
                    if let Some(previous_max_variance) =
                        routers[source_router_index].get_max_variance_before_round()
                    {
                        routers[lca].set_max_variance_before_round(previous_max_variance);
                    }
                    let improve_queue_clone_source: Vec<_> = {
                        let src_queue = routers[source_router_index].get_improve_queue();
                        src_queue
                            .iter()
                            .map(|(value, sell_info)| (value.clone(), sell_info.clone()))
                            .collect()
                    };

                    let improve_queue_of_lca = routers[lca].get_improve_queue();
                    improve_queue_of_lca.extend(improve_queue_clone_source);
                }
            }
        }

        routers[source_router_index].remove_device(device);
        routers[dist_router_index].add_device(device);
    }
    for i in 0..routers.len() {
        if i == dist || i == source_router_index {
            routers[i].set_is_updated(true);
        } else {
            routers[i].set_is_updated(false);
        }

        routers[i].set_is_max_variance_router(false);
        //if routers[i].get_router().index() == 245{
        //     println!("245 device: {}", routers[i].get_devices().len());
        //     println!("Before delete queue len: {}", routers[i].get_improve_queue().len());
        //}
        routers[i]
            .get_improve_queue()
            .retain(|(_value, sell_info)| {
                let is_delete_on_source_router_id =
                    sell_info.get_router().index() != source_router_index;
                let is_delete_on_dist_router_id =
                    sell_info.get_router().index() != dist_router_index;

                is_delete_on_dist_router_id && is_delete_on_source_router_id
            });

        if dist == 0 && source == 0 && device_no == 0 {
            routers[i].set_optimal(true);
        }
    }

    println!("{}, {}, {}", dist, source, device_no);
}

fn get_node_info(node_id: usize) -> Option<(usize, usize, Option<usize>, usize)> {
    if node_id >= 1190 {
        return None;
    }

    let levels = [
        (0, 13),   // level 0
        (13, 52),  // level 1
        (65, 208), // level 2
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
