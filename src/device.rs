use petgraph::graph::NodeIndex;

#[derive(Clone)]
pub struct Device {
    id: NodeIndex<u32>,
    time: f64,
}

impl Device {
    pub fn new(id: NodeIndex, time: f64) -> Self {
        Self { id, time }
    }

    pub fn get_id(&self) -> NodeIndex {
        self.id
    }

    pub fn get_learning_time(&self) -> f64 {
        self.time
    }
}
