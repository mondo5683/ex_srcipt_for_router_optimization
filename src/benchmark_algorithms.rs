use crate::{device::Device, router::Router};
use clustering::*;
use petgraph::graph::NodeIndex;
use std::collections::HashMap;

pub fn clustering_device<'a>(
    device_list: &'a Vec<Device>,
    k: usize,
    max_iter: usize,
) -> Clustering<'a, Device> {
    let clustering = kmeans(k, &device_list, max_iter);

    println!("membership: {:?}", clustering.membership);
    //println!("centroids : {:?}", clustering.centroids);

    clustering
}

pub fn generate_hashmap(
    time_exec: Vec<f64>,
    nodex_index: &Vec<NodeIndex<u32>>,
    membership: Vec<usize>,
) -> HashMap<usize, Vec<Device>> {
    // グループ化
    let mut groups: HashMap<usize, Vec<Device>> = HashMap::new();
    for i in 0..time_exec.len() {
        let device = Device::new(nodex_index[i], time_exec[i]);
        groups.entry(membership[i]).or_insert(vec![]).push(device);
    }

    groups
}
pub fn calculate_variances_clusters(clustered_group: &HashMap<usize, Vec<Device>>) -> Vec<f64> {
    let mut variance_vec = Vec::<f64>::new();
    // 分散を計算
    for (_group, values) in clustered_group.iter() {
        let mean =
            values.iter().map(|d| d.get_learning_time()).sum::<f64>() / (values.len() as f64);
        let variance = values
            .iter()
            .map(|v| (v.get_learning_time() - mean).powi(2))
            .sum::<f64>()
            / (values.len() as f64);
        variance_vec.push(variance);
        //println!("Group {}: Variance = {}", _group, variance);
    }

    variance_vec
}

//TODO その後各クラスターの最適なルータを検索できる
pub fn allocate_cluster_to_optimal_router(
    routers: &mut Vec<Router>,
    clustered_group: HashMap<usize, Vec<Device>>,
    _max_variance: f64,
) {
    //println!("Max Variance: {}", _max_variance);

    //initialize device_list for each routers
    for r in routers.iter_mut() {
        r.set_devices(Vec::<(NodeIndex, f64)>::new())
    }

    let mut hashmap_clusterid_to_router_index: HashMap<usize, Vec<(NodeIndex, f64)>> =
        HashMap::new();

    for (clusterid, devices) in clustered_group.iter() {
        let device_list: Vec<(NodeIndex, f64)> = devices
            .iter()
            .map(|a| (a.get_id(), a.get_learning_time()))
            .collect();

        for i in 0..routers.len() {
            let (communication_reduction, _) =
                routers[i].calc_router_value_return_two_variables(&device_list);
            //println!("communication_reduction: {}",communication_reduction);
            insert_and_trim(
                &mut hashmap_clusterid_to_router_index,
                *clusterid,
                (routers[i].get_router(), communication_reduction),
            );
        }
    }

    //いまのところすべてのルータをベクタに格納，TODO メモリ的にTOP-Kにしないといけないかも
    for vec in hashmap_clusterid_to_router_index.values_mut() {
        vec.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    }

    let mut num_of_not_allocated_clusters = clustered_group.len();
    while num_of_not_allocated_clusters > 0 {
        //println!("{:?}",hashmap_clusterid_to_router_index);
        //Allocate cluster to router by Greedy Algorithms
        //max_network_cost_redecutionを計算
        let mut max_network_cost_redecution = -99999999999.0;
        let mut cluster_assinged_router_id = 0;
        let mut assined_cluster_id = 0;
        for (clusterid, reduction_vec) in hashmap_clusterid_to_router_index.iter() {
            let max_network_cost_reduction_in_cluster: f64 = reduction_vec[0].1;

            if max_network_cost_redecution < max_network_cost_reduction_in_cluster {
                max_network_cost_redecution = max_network_cost_reduction_in_cluster;
                cluster_assinged_router_id = reduction_vec[0].0.index();
                assined_cluster_id = *clusterid;
                //println!("max_network_cost_redecution:{}, cluster_assinged_router_id: :{}, assined_cluster_id: {}", max_network_cost_redecution,cluster_assinged_router_id, assined_cluster_id);
            }
        }

        //routerにデバイスをallocate
        for device in clustered_group.get(&assined_cluster_id).iter() {
            for d in device.iter().map(|a| (a.get_id(), a.get_learning_time())) {
                routers[cluster_assinged_router_id].add_device(d);
            }
        }

        //割り当てたclusterのエントリをhashmapより削除
        hashmap_clusterid_to_router_index.remove(&assined_cluster_id);

        //割り当てられたrouterのエントリを各hashmapより削除
        for (_, reduction_vec) in hashmap_clusterid_to_router_index.iter_mut() {
            reduction_vec.retain(|x| x.0.index() != cluster_assinged_router_id);
        }

        num_of_not_allocated_clusters -= 1;
    }

    //    println!("{:?}",hashmap_clusterid_to_router_index);
}

fn insert_and_trim(
    map: &mut HashMap<usize, Vec<(NodeIndex, f64)>>,
    key: usize,
    value: (NodeIndex, f64),
) {
    // Entry APIを使うと便利
    let vec = map.entry(key).or_insert_with(Vec::new);
    vec.push(value);

    if vec.len() > 10 {
        // 降順（大きい順）にソートして最初の10個だけ残す
        vec.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        vec.truncate(10);
    }
}
