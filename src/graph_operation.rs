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

pub fn minimum_cost_to_ps(ps: &NodeIndex, vertex: &NodeIndex, gr:&UnGraph<String,String>) -> Option<i32> {

    let res = dijkstra(gr, *vertex, None,|_| 1);
    

    for (key, value) in res{
        if key == *ps{
            return Some(value);
        }
    }
    return None;

}
