use std::collections::HashMap;

use crate::graph_operation::minimum_hop_to_node;
use crate::{sell_info::{SellInfo}, ALPHA, BETA};
use petgraph::{graph::NodeIndex, Graph,Undirected};
use std::io::{Write, Result};

fn median(v: &Vec<f64>) -> f64 {
    let mut s = v.clone();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let len = s.len();
    if len % 2 == 1 {
        s[len / 2]
    } else {
        (s[len / 2 - 1] + s[len / 2]) / 2.0
    }

}

pub struct Router<'a> {
    previous_obvalue: f64,
    router_obvalue: f64,
    devices: Vec::<(NodeIndex,f64)>,
    optimal: bool,
    topo: &'a Graph::<String,String,Undirected>,
    model_size: f64,
    router: &'a NodeIndex,
    ps:     &'a NodeIndex,
    variance_vec: HashMap::<NodeIndex,f64>,
    is_updated_in_previous_round: bool,
    improved_queue: Vec<(f64,SellInfo)>
}

impl<'a> Router<'a> {
    pub fn new(topo: &'a Graph::<String, String, Undirected>, router:&'a NodeIndex, ps:&'a NodeIndex,model_size:f64) -> Self {
        Self {
            previous_obvalue: 0.0, 
            router_obvalue: 0.0,
            devices: Vec::<(NodeIndex,f64)>::new(),
            model_size,
            topo,
            ps,
            router,
            optimal: false, 
            variance_vec: HashMap::new(),
            //To make routers conduct first sell info process, we initialize is_updated true
            is_updated_in_previous_round: true,
            improved_queue: Vec::<(f64, SellInfo)>::new()
        }
    }
    pub fn get_optimal(&self) -> bool {
        self.optimal
    }
    pub fn set_optimal(&mut self, b:bool) {
        self.optimal = b;
    }
    pub fn get_devices(&self) -> Vec::<(NodeIndex,f64)>{
        self.devices.clone()
    }
    pub fn get_router(&self) -> NodeIndex {
        *self.router
    }
    pub fn get_previous_obvalue(&self) -> f64{
        self.previous_obvalue
    }
    pub fn get_router_value(&self) -> f64 {
        self.router_obvalue
    }
    pub fn set_devices(&mut self, device_list:Vec<(NodeIndex, f64)>) {
        self.devices = device_list;
    }
    pub fn set_variance_vec(&mut self, hashmap: HashMap::<NodeIndex,f64>){
        self.variance_vec = hashmap;
    }
    pub fn get_variance_vec(&self) -> HashMap::<NodeIndex,f64>{
        self.variance_vec.clone()
    }
    pub fn set_is_updated(&mut self, is_updated: bool) {
        self.is_updated_in_previous_round = is_updated;
    }
    pub fn get_is_updated(&self) -> bool{
        self.is_updated_in_previous_round
    }
    pub fn insert_to_improve_queue(&mut self, value:f64, sell_info: SellInfo) {
        self.improved_queue.push( (value, sell_info) );
    }
    pub fn get_improve_queue(&mut self) -> &mut Vec<(f64,SellInfo)>{
        &mut self.improved_queue
    }

    pub fn print_inner_cluster_devices(&self, writer: &mut impl Write, device_index_offset:usize) -> Result<()> {
        write!(writer, "\t\"{}\": [", self.router.index())?;
    
        let last_index = self.devices.len().saturating_sub(1);
        for (i, d) in self.devices.iter().enumerate() {
            let device_index = d.0.index() - device_index_offset;
            if i == last_index {
                write!(writer, "{}", device_index)?;
            } else {
                write!(writer, "{},", device_index)?;
            }
        }
    
        write!(writer, "]")?;
        Ok(())
    }
    
    pub fn hop_to_node(&self, target_node_index: &NodeIndex) -> usize {
        match  minimum_hop_to_node(target_node_index, &self.router, &self.topo) {
            Some(hop_to_ps) => hop_to_ps,
            None => 0
        }

    }
    
    pub fn add_device(&mut self, device:(NodeIndex,f64)){
        self.devices.push(device);
    }
    
    pub fn add_device_from_sellinfo(&mut self, sell_info:SellInfo){
        if sell_info.get_router() == *self.router {
            self.optimal = true;
        }
        self.devices.push(sell_info.get_device());
    }

    pub fn sell_all_device(&mut self) -> Vec<SellInfo> {
        
        let mut sell_info_list = Vec::<SellInfo>::new();
        let device_list = self.devices.clone();

        for d in device_list{
            let (network_traffic, before_variance, variance, router_value) = self.assume_remove_device(d);
            let sell_info = SellInfo::new(d, self.get_router(), self.router_obvalue, router_value, variance, before_variance, network_traffic);
            sell_info_list.push(sell_info);
        }

        sell_info_list

    }

    pub fn assume_add_device(&self, device: (NodeIndex, f64), before_variance:f64, after_variance:f64) -> (f64,f64){

//        if self.devices.len() < 1 {
//            return INF;
//        }
        //更新した相手が最大だった場合，更新後のvarianceではなくて変更後のvarianceをとってくる必要がある 
        let mut assume_device_list = self.devices.clone();
        let mut variance_vec = self.variance_vec.clone(); 
        assume_device_list.push(device);

        let (network_traffic, variance) = self.calc_router_value_return_two_variables(&assume_device_list);
        let (_ ,max_variance_from_hasmap) = variance_vec.iter().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).unwrap();

        let max_variance_of_others = if *max_variance_from_hasmap == before_variance{
            variance_vec.retain(|_key, value| *value != before_variance);
            let (_ ,max_variance_from_hasmap) = variance_vec.iter().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).unwrap();
            after_variance.max(*max_variance_from_hasmap)
        }else { *max_variance_from_hasmap};

        if max_variance_of_others > variance{
            (ALPHA * network_traffic - BETA * max_variance_of_others, max_variance_of_others)
        } else {
            let router_value = ALPHA * network_traffic - BETA * variance;
            (router_value, variance)
        }
        
    }
    pub fn assume_remove_device(&mut self, device: (NodeIndex, f64)) -> (f64,f64,f64,f64){

//        if self.devices.len() < 1 {
//            return INF;
//        }
        let mut device_list = self.devices.clone();
        let ( _ , before_variance) = self.calc_router_value_return_two_variables(&device_list);
        device_list.retain(|&x| x != device);
        let (network_traffic, variance) = self.calc_router_value_return_two_variables(&device_list);
        
        let (router ,max_variance_from_hasmap) = self.variance_vec.iter().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).unwrap();

        if *max_variance_from_hasmap > variance{
            if *router == self.get_router(){
                (network_traffic, before_variance, variance, ALPHA * network_traffic - BETA * variance)
            } else {
                (network_traffic, before_variance, variance, ALPHA * network_traffic - BETA * *max_variance_from_hasmap)
            }
        } else {
            (network_traffic, before_variance, variance, ALPHA * network_traffic - BETA * variance)
        }
        
    }
    pub fn update_obvalue(&mut self) -> (f64,f64){
        
        let device_list = self.devices.clone();
        let (network_traffic, variance) = self.calc_obvalue_return_two_variables(&device_list);
        
        self.previous_obvalue =   ALPHA * network_traffic + BETA * variance;

//        println!("network traffic: {}, variance: {}", network_traffic, variance);
//        println!("objective value : {}",self.previous_obvalue);
        
        (network_traffic,variance)
    }
    pub fn update_router_value(&mut self) -> (f64, f64){
        let device_list = self.devices.clone();
        let (network_traffic, variance) = self.calc_router_value_return_two_variables(&device_list);
        

        if let Some((_ ,max_variance_from_hasmap)) = self.variance_vec.iter().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()){
            if *max_variance_from_hasmap > variance{
                //println!("Use max variance value: {}",*max_variance_from_hasmap);
                self.router_obvalue =   ALPHA * network_traffic - BETA * *max_variance_from_hasmap;
            } else {
                //println!("Use self variance value: {}, {}",network_traffic,variance);
                self.router_obvalue = ALPHA * network_traffic - BETA * variance;
            }
        }else {
            self.router_obvalue = ALPHA * network_traffic - BETA * variance;
        }

        
        (network_traffic, variance)
//        println!("network traffic: {}, variance: {}", network_traffic, variance);
//        println!("objective value : {}",self.previous_obvalue);
        

    }

    pub fn remove_device(&mut self, device:(NodeIndex,f64)){
        self.devices.retain(|&x| x != device);
    }

    pub fn calc_obvalue(&mut self, device_list: Vec::<(NodeIndex,f64)>) -> f64{

//        println!("---------------Start Calc obvalue------\n");

        if device_list.len() <= 0{
            return 0.0;
        }

        let device_num = device_list.len();


        let mut before_merge_hopping = 0;
        let mut after_merge_hopping = 0;
        let device_cluster: Vec<NodeIndex> = device_list.iter().map(|(x,_)| *x).collect();

//        println!("{:?}", device_cluster);
            

        for device in device_cluster{

/*
           match minimum_cost_to_ps(self.ps, &device, self.topo){
               Some(cost) => {
//                   println!("minimum_cost_to_ps:{}",cost);
                   overall_hopping += cost;
               },                     
               None => println!("Can't go to {:?}", device),
           } 
*/
           let start = device;
           let goal = *self.router;
           if let Some(hop_to_router_from_device) = minimum_hop_to_node(&goal, &start, self.topo){
                before_merge_hopping += hop_to_router_from_device;
           }
//               println!("Before Merge Path: {:?}", p);

           let start = *self.router;
           let goal = *self.ps;

           if let Some(hop_to_ps_from_router) = minimum_hop_to_node(&goal, &start, self.topo){
                after_merge_hopping = hop_to_ps_from_router;
           }

        }
        
//        let before_merge_cost = self.model_size * overall_hopping as f64;
        let after_merge_cost = self.model_size * (after_merge_hopping + before_merge_hopping) as f64;
        let network_traffic = after_merge_cost;
//        println!("network packet amount cost: {}", network_traffic);
 
        let device_time_list: Vec<f64> = device_list.iter().map(|(_,x)| *x).collect();
        let time_exec_average = device_time_list.iter().sum::<f64>() as f64 / device_time_list.len() as f64 ;
        let mut variance = 0.0;

        if device_list.len() <= 1 {
            variance = time_exec_average;
        } else {
            for time in device_time_list {
                let std = (time as f64 - time_exec_average) * (time as f64 - time_exec_average)
                            / device_num as f64;
                variance = variance + std;
            
            }

//            println!("Variance: {}", variance);
        }
        let obvalue = ALPHA * network_traffic + BETA * variance;
//        println!("objective value : {}",obvalue);

//        println!("-------------------------------------------\n\n");
        

        obvalue
   
    }


    pub fn calc_obvalue_return_two_variables(&mut self, device_list: &Vec::<(NodeIndex,f64)>) -> (f64,f64){

        if device_list.len() <= 0{
            return (0.0,0.0);
        }

        let device_num = device_list.len();


        let mut before_merge_hopping = 0;
        let mut after_merge_hopping = 0;
        let device_cluster: Vec<NodeIndex> = device_list.iter().map(|(x,_)| *x).collect();

            

        for device in device_cluster{

/*
           match minimum_cost_to_ps(self.ps, &device, self.topo){
               Some(cost) => {
//                   println!("minimum_cost_to_ps:{}",cost);
                   overall_hopping += cost;
               },                     
               None => println!("Can't go to {:?}", device),
           } 
*/
           let start = device;
           let goal = *self.router;
           if let Some(hop_to_router_from_device) = minimum_hop_to_node(&goal, &start, self.topo){
                before_merge_hopping += hop_to_router_from_device;
           }
//               println!("Before Merge Path: {:?}", p);

           let start = *self.router;
           let goal = *self.ps;

           if let Some(hop_to_ps_from_router) = minimum_hop_to_node(&goal, &start, self.topo){
                after_merge_hopping = hop_to_ps_from_router; 
           }
 
//         println!("After Merge Path: {:?}", p);

        }
        
        //let before_merge_cost = self.model_size * overall_hopping as f64;
        let after_merge_cost = self.model_size * (after_merge_hopping + before_merge_hopping) as f64;
        let network_traffic = after_merge_cost;
 
        let device_time_list: Vec<f64> = device_list.iter().map(|(_,x)| *x).collect();
       
        let time_exec_average = device_time_list.iter().sum::<f64>() as f64 / device_time_list.len() as f64 ;
        let mut variance = 0.0;

        if device_list.len() <= 1 {
            variance = time_exec_average;
        } else {
            for time in device_time_list {
                let std = (time as f64 - time_exec_average) * (time as f64 - time_exec_average)
                            / device_num as f64;
                variance = variance + std;
            
            }

        }
        
/*
        let min = device_time_list.iter().cloned().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let max = device_time_list.iter().cloned().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();

        let diff = max - min;
        let variance = diff * diff;
*/
        (network_traffic ,variance)
   
    }


    pub fn calc_router_value_return_two_variables(&self, device_list: &Vec::<(NodeIndex,f64)>) -> (f64,f64){

        if device_list.len() <= 0{
            return (0.0,0.0);
        }

        let device_num = device_list.len();


        let mut before_merge_hopping = 0;
        let mut after_merge_hopping = 0;
        let mut overall_hopping = 0;
        let device_cluster: Vec<NodeIndex> = device_list.iter().map(|(x,_)| *x).collect();

            

        for device in device_cluster{


           match minimum_hop_to_node(self.ps, &device, self.topo){
               Some(cost) => {
//                   println!("minimum_cost_to_ps:{}",cost);
                   overall_hopping += cost;
               },                     
               None => println!("Can't go to {:?}", device),
           } 

                      let start = device;
           let goal = *self.router;
           if let Some(hop_to_router_from_device) = minimum_hop_to_node(&goal, &start, self.topo){
                before_merge_hopping += hop_to_router_from_device;
           }
//               println!("Before Merge Path: {:?}", p);

           let start = *self.router;
           let goal = *self.ps;

           if let Some(hop_to_ps_from_router) = minimum_hop_to_node(&goal, &start, self.topo){
                after_merge_hopping = hop_to_ps_from_router;
           }

        }
        
        let before_merge_cost = self.model_size * overall_hopping as f64;
        let after_merge_cost = self.model_size * (after_merge_hopping + before_merge_hopping) as f64;
        let network_traffic = before_merge_cost - after_merge_cost;
 
        let device_time_list: Vec<f64> = device_list.iter().map(|(_,x)| *x).collect();
        
        let time_exec_average = device_time_list.iter().sum::<f64>() as f64 / device_time_list.len() as f64 ;
        let mut variance = 0.0;

        if device_list.len() <= 1 {
            variance = time_exec_average;
        } else {
            for time in device_time_list {
                let std = (time as f64 - time_exec_average) * (time as f64 - time_exec_average)
                            / device_num as f64;
                variance = variance + std;
            
            }

        }
        
        //TODO CHANGE いったん分散をクラスタごとの最小値と最大値の差の二乗に変えてみる
/*
        let min = device_time_list.iter().cloned().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let max = device_time_list.iter().cloned().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();

        let diff = max - min;
        let variance = diff * diff;
*/
        (network_traffic ,variance)
   
    }

}
