use petgraph::{algo::dijkstra,graph::{NodeIndex, UnGraph}};


pub fn _divide_path(path: &Vec::<NodeIndex>,pivot:NodeIndex) 
    -> Option<(Vec::<NodeIndex>, Vec<NodeIndex>)>{
     
    let mut previous = Vec::<NodeIndex>::new();
    let mut merged = Vec::<NodeIndex>::new();
    let mut flag = false;

    for vertex in path{
        if flag == true{
            merged.push(*vertex);
        } else {
            if *vertex == pivot {
               flag = true;
               previous.push(*vertex);
               merged.push(*vertex);
            }else{
                previous.push(*vertex);
            }
        }
    }

    if flag == false { return None;}

    return Some((previous, merged));
}

pub fn minimum_hop_to_node(target_node_index: &NodeIndex, source_node_index: &NodeIndex, gr:&UnGraph<String,String>) -> Option<usize> {

    let res = dijkstra(gr, *source_node_index, None,|_| 1);

    if res.contains_key(target_node_index){
        let hop_num = *res.get(target_node_index).unwrap() as usize; 
        println!("Cost {:?} to {:?}: {}", *source_node_index, *target_node_index, hop_num );
        Some(hop_num)
    }
    else { 
        println!("Unreachable from {:?} to {:?}", *source_node_index,*target_node_index);
        None }

}
