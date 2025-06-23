use petgraph::graph::NodeIndex;

pub struct SellInfo{
    device: (NodeIndex, f64),
    previous_obvalue: f64,
    after_remove_obvalue: f64,
    router: NodeIndex,
    variance: f64,
    network_cost: f64,

}

impl SellInfo {
    pub fn new(device:(NodeIndex, f64), router: NodeIndex,previous_obvalue:f64, after_remove_obvalue:f64, variance:f64, network_cost:f64) -> Self {
        Self {
            device,
            router,
            previous_obvalue,
            after_remove_obvalue,
            variance,
            network_cost,
        }

    }
    pub fn clone(&self) -> Self{
        Self {
            device: self.device.clone(),
            previous_obvalue: self.previous_obvalue.clone(),
            after_remove_obvalue: self.after_remove_obvalue.clone(),
            router: self.router.clone(),
            variance: self.variance.clone(),
            network_cost: self.network_cost.clone(),
        }
    }
    pub fn get_device(&self) -> (NodeIndex,f64) {
        self.device
    }
    
    pub fn get_router(&self) -> NodeIndex {
        self.router
    }
    pub fn get_previous_obvalue(&self) -> f64 {
        self.previous_obvalue
    }
    pub fn get_after_remove_cost(&self) -> f64 {
        self.after_remove_obvalue
    }
    pub fn get_variance(&self) -> f64 {
        self.variance
    }
    pub fn get_network_cost(&self) -> f64 {
        self.network_cost
    }

    pub fn view_sell_info(&self) -> String {
        let device_s =  format!("device: {:?}\n",self.device);
        
        format!("{}previous_obvalue: {}\nafter_remove_obvalue: {}\nrouter: {:?}",
                device_s,self.previous_obvalue, self.after_remove_obvalue,self.router)
    }
}

pub fn max_obvalue_from_sell_info(device_list: &Vec::<SellInfo>) -> f64{
    
    let mut max_obvalue = 0.0;
    
    for d in device_list.iter() {
       if max_obvalue < d.previous_obvalue{
            max_obvalue = d.previous_obvalue;
       }
    }

    max_obvalue
}
