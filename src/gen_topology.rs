use petgraph::{graph::NodeIndex, Graph, Undirected};
use crate::Arc;
use nalgebra as na;

pub fn generate_router_array( rt: &na::DMatrix<i32>, topology: &mut Graph::<String,String,Undirected>) -> Vec::<NodeIndex> {
    
    let mut r = Vec::<NodeIndex>::new();

    for i in 0..rt.nrows(){
        let s : String = "router".to_string() + &i.to_string();
        r.push(topology.add_node(s));
    }
    r
}

pub fn generate_device_array( e: &na::DMatrix<i32>, topology: &mut Graph::<String,String,Undirected>) -> Vec::<NodeIndex> {
    
    let mut d = Vec::<NodeIndex>::new();

    for i in 0..e.ncols(){
        let s : String = "device".to_string() + &i.to_string();
        d.push(topology.add_node(s));
    }
    d
}

pub fn generate_edge_array( rt: &na::DMatrix<i32>, e: &na::DMatrix<i32>, 
                                r: &Vec<NodeIndex>, d:&Vec<NodeIndex>)
    -> Vec<(NodeIndex,NodeIndex)>{
    let mut edge_array = Vec::<(NodeIndex,NodeIndex)>::new();
    
    //TODO もしPSを複数のrouterとつなげる場合ここを変更する
//    edge_array.push((*ps,r[0]));

    //generate router-router edges
    for column in 0..rt.ncols(){
        for row in 0..rt.nrows(){
            if rt[(row,column)] == 1{
                let edge = (r[row], r[column]);
                edge_array.push(edge);
            }
        }
    }
    //generate router-device edges
    for column in 0..e.ncols(){
        for row in 0..e.nrows(){
            if e[(row,column)] == 1{
                let edge = (r[row], d[column]);
                edge_array.push(edge);
            }
        }
    }
    edge_array

}
