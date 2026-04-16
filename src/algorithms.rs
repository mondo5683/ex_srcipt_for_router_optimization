use std::collections::HashMap;
use std::ops::Index;

use petgraph::graph::{Node, NodeIndex};

use crate::imp_info::ImpInfo;
use crate::router::Router;
use crate::sell_info::SellInfo;
use crate::{Device, ALPHA, BETA};
use std::time;

pub fn optimize_obvalue_in_cluster(routers: &mut Vec<Router>, is_first_round:bool) {
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

    let mut before_max_variance = 0.0;

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

        let (router, max_variance_from_hasmap) = variance_hashmap
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .unwrap();
        println!(
            "max_variance: {}, router_id: {}",
            *max_variance_from_hasmap,
            router.index()
        );

        //TO Interrupt for Experiments
        if is_first_round {
            if *max_variance_from_hasmap == before_max_variance {
                break;
            }
        }
        before_max_variance = *max_variance_from_hasmap;

        let sell_info_list = generate_sell_info_list(routers);
        update_routers_sell_info_list(
            routers,
            &sell_info_list,
            &variance_hashmap,
            max_variance_from_hasmap,
        );
        //let variance_candidates = routers[max_variance_router.index()].find_new_candidates();
        // let max_imp_new_routers = propose_new_pair_by_new_routers(routers, variance_candidates);
        let max_imp_exsiting_routers = search_maximized_swap_node(routers);
        swap_device_between_routers(routers, &max_imp_exsiting_routers);

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
        r.alpha() * si.get_network_cost()
            - r.beta() * max_variance_from_others
            - si.get_previous_obvalue()
    };

    (
        buyer_improve_value,
        seller_improve_value,
        add_assume_variance,
    )
}
/*
fn propose_new_pair_by_new_routers(
    routers: &mut Vec<Router>,
    variance_candidates: Vec<(NodeIndex, f64)>,
) -> Option<ImpInfo> {
    let mut new_routers_impinfo: Option<ImpInfo> = None;

    //はじめに売り手の改善値を出す。

    let seller_improve_value = 1; //improve_queueより取ってくる
    for r in routers.iter_mut() {
        if r.get_devices().iter().len() > 0 {
            continue;
        }

        let buyer_improve_value = router_add_assume();
    }

    new_routers_impinfo = Some(max_imp_value);

    new_routers_impinfo
}
*/
fn search_maximized_swap_node(routers: &mut Vec<Router>) -> (Option<ImpInfo>) {
    let mut max_value = 0.0;
    let mut max_imp_info: Option<ImpInfo> = None;

    let mut now: Option<time::Instant> = None;
    let mut is_first_router = true;

    for r in routers.iter_mut() {
        if r.get_devices().len() < 1 {
            //新しいペアを見つける処理を書く
            r.set_optimal(true);
            if r.get_router().index() != 0 {
                continue;
            }
        }
        //r.find_new_candidates();

        /*処理時間計測用
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
        */

        //r.get_improve_queue().retain(|&(value ,_ )| value > -600.0 );
        let _r_index = r.get_router();
        match r.get_improve_queue().iter().max_by(|a, b| {
            a.0.get_improve_value()
                .partial_cmp(&b.0.get_improve_value())
                .unwrap()
        }) {
            Some(result) => {
                let (queue_data, sell_info) = result;

                let max_gain_for_this_router = queue_data.get_improve_value();
                if max_value < max_gain_for_this_router {
                    let improve_value = max_gain_for_this_router;
                    let mut max_candidate_devices: Vec<Device> = Vec::new();
                    let mut send_router: Vec<NodeIndex> = Vec::new();
                    send_router.push(sell_info.get_router());
                    max_candidate_devices.push(Device::new(
                        sell_info.get_device().0,
                        sell_info.get_device().1,
                    ));

                    let max_move_device_info = ImpInfo::new(
                        max_candidate_devices,
                        _r_index,
                        send_router,
                        improve_value,
                        false,
                    );
                    max_value = max_gain_for_this_router;
                    max_imp_info = Some(max_move_device_info);
                }
            }
            None => {
                r.set_optimal(true);
            }
        }
        if r.get_router().index() == 1 {
            if let Some(t) = now {
                println!("process time {:?}", t.elapsed());
            }
        }
    }

    (max_imp_info)
}

fn swap_device_between_routers(routers: &mut Vec<Router>, max_imp_info: &Option<ImpInfo>) {
    let mut dist: usize = 0;
    let mut dist_router_index: usize = 0;
    let mut source: usize = 0;
    let mut source_router_index: usize = 0;
    let mut device_no: usize = 0;
    let mut moved_device: Option<Device> = None;

    let routers_index_map: HashMap<NodeIndex, usize> = routers
        .iter()
        .enumerate()
        .map(|(i, r)| (r.get_router(), i))
        .collect();

    if let Some(imp_info) = max_imp_info {
        if !imp_info.is_pair() {
            let device = imp_info.devices()[0].clone();

            let mut nodeindex = &imp_info.receive_router();
            if let Some(&index) = routers_index_map.get(nodeindex) {
                dist = nodeindex.index();
                device_no = device.get_id().index();
                dist_router_index = index;
            }
            let mut nodeindex = &imp_info.send_routers()[0];
            if let Some(&index) = routers_index_map.get(nodeindex) {
                //         //println!("{:?}",devices);
                //         for device in devices {
                source = nodeindex.index();
                source_router_index = index;
                moved_device = Some(device);
                //         }
                //     }
            }
        }
    };

    if let Some(device) = moved_device {
        //LCAを検出しそこに配置
        if let Some(lca) = find_lca(dist, source) {
            if !(lca.eq(&source) || lca.eq(&dist)) {
                println!(
                    "Redunduncy Detected!! Device:{} will be allocated to Router: {}",
                    device.get_id().index(),
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

        routers[source_router_index].remove_device((device.get_id(), device.get_learning_time()));
        routers[dist_router_index].add_device((device.get_id(), device.get_learning_time()));
    }
    for i in 0..routers.len() {
        if i == dist || i == source_router_index {
            routers[i].set_is_updated(true);
        } else {
            routers[i].set_is_updated(false);
        }

        routers[i].set_is_max_variance_router(false);
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

fn update_routers_sell_info_list(
    routers: &mut Vec<Router>,
    sell_info_list: &Vec<SellInfo>,
    variance_hashmap: &HashMap<NodeIndex, f64>,
    max_variance_from_hashmap: &f64,
) {
    for r in routers.iter_mut() {
        let mut results = Vec::new();
        for (index, (_, sell_info)) in r.get_improve_queue().iter_mut().enumerate() {
            // 必要データを clone
            let sell_info_copy = sell_info.clone(); // Clone が必要
            results.push((index, sell_info_copy));
        }

        // improve_queue 内の該当する improve_value を更新
        // 始めに自身の持つsell_info_listに直前のトレードを反映し更新させる
        let mut decreased_variance = 0.0;
        for (index, sell_info_copy) in results.iter_mut() {
            //一回目のtrade roundでないなら更新する必要がある（このifの処理が実行される）
            if let Some(max_variance_before_round) = r.get_max_variance_before_round() {
                let previous_obvalue = sell_info_copy.get_previous_obvalue();
                //各sell_infoのvariance部分を改善した分下げる
                decreased_variance = max_variance_before_round - *max_variance_from_hashmap;
                r.get_improve_queue()[*index]
                    .1
                    .set_previous_obvalue(previous_obvalue + BETA * decreased_variance);
                sell_info_copy.set_previous_obvalue(previous_obvalue + BETA * decreased_variance);

                //updateする対象がmax_varianceを持つ対象の場合そのまま更新
                if *max_variance_from_hashmap == sell_info_copy.get_before_remove_variance() {
                    let network_cost = sell_info_copy.get_network_cost();
                    let after_remove_variance = sell_info_copy.get_after_remove_variance();
                    let up_to_date_after_remove_obvalue =
                        r.alpha() * network_cost - r.beta() * after_remove_variance;
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

            // For acceleralating process, Skip calc_improvement_using_si_for_router() if it's unnessesary
            let before_add_assume_variance =
                r.get_improve_queue()[*index].0.get_assume_add_variance();
            let is_variance_checked = r.get_improve_queue()[*index].0.is_variance_checked();

            //もし自分が前のラウンドでやり取りをしていたらsell_info_listをすべて更新する
            let (buyer_improve_value, seller_improve_value, add_assume_variance) = if r
                .get_is_updated()
            {
                r.get_improve_queue()[*index].0.set_variance_checked(false);
                calc_improvement_using_si_for_router(r, &sell_info_copy, variance_hashmap)
            } else if before_add_assume_variance > *max_variance_from_hashmap {
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
            //前のラウンドでデバイスを交換していないなら更新をサボるようにする
            } else {
                let before_variance = sell_info_copy.get_before_remove_variance();
                let (buyer_improve_value, seller_improve_value) = if before_variance
                    == *max_variance_from_hashmap
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
        }

        //新しく発行されたsell_infoを取り込む処理
        for si in sell_info_list.iter() {
            if si.get_router() == r.get_router() {
                continue;
            }

            let (buyer_improve_value, seller_improve_value, add_assume_variance) =
                calc_improvement_using_si_for_router(r, si, &variance_hashmap);

            let improvement_for_si = buyer_improve_value + seller_improve_value;

            r.insert_to_improve_queue(
                improvement_for_si,
                buyer_improve_value,
                seller_improve_value,
                add_assume_variance,
                si.clone(),
            );
        }
        r.set_max_variance_before_round(*max_variance_from_hashmap);
    }
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
