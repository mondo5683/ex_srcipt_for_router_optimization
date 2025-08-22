use petgraph::graph::NodeIndex;

pub struct SellInfo{
    device: (NodeIndex, f64),
    previous_obvalue: f64,
    after_remove_obvalue: f64,
    router: NodeIndex,
    after_remove_variance: f64,
    before_remove_variance: f64,
    network_cost: f64,

}

impl SellInfo {
    pub fn new(device:(NodeIndex, f64), router: NodeIndex,previous_obvalue:f64, after_remove_obvalue:f64, after_remove_variance:f64, 
               before_remove_variance:f64, network_cost:f64) -> Self {
        Self {
            device,
            router,
            previous_obvalue,
            after_remove_obvalue,
            after_remove_variance,
            before_remove_variance,
            network_cost,
        }

    }
    pub fn clone(&self) -> Self{
        Self {
            device: self.device.clone(),
            previous_obvalue: self.previous_obvalue.clone(),
            after_remove_obvalue: self.after_remove_obvalue.clone(),
            router: self.router.clone(),
            before_remove_variance: self.before_remove_variance.clone(),
            after_remove_variance: self.after_remove_variance.clone(),
            network_cost: self.network_cost.clone(),
        }
    }
    pub fn print_sell_info(&self) {
        println!("SellInfo:");
        println!("  device: (NodeIndex: {}, value: {})", self.device.0.index(), self.device.1);
        println!("  router: {}", self.router.index());
        println!("  previous_obvalue: {}", self.previous_obvalue);
        println!("  after_remove_obvalue: {}", self.after_remove_obvalue);
        println!("  before_remove_variance: {}", self.before_remove_variance);
        println!("  after_remove_variance: {}", self.after_remove_variance);
        println!("  network_cost: {}", self.network_cost);
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
    pub fn get_before_remove_variance(&self) -> f64 {
        self.before_remove_variance
    }
    pub fn get_after_remove_variance(&self) -> f64 {
        self.after_remove_variance
    }
    pub fn get_network_cost(&self) -> f64 {
        self.network_cost
    }
    pub fn set_previous_obvalue(&mut self, previous_obvalue:f64) {
        self.previous_obvalue = previous_obvalue;
    }
    pub fn set_after_remove_obvalue(&mut self, after_remove_obvalue:f64) {
        self.after_remove_obvalue = after_remove_obvalue;
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
