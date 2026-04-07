use crate::Device;
use petgraph::graph::NodeIndex;

#[derive(Clone)]
pub struct ImpInfo {
    devices: Vec<Device>,
    receive_router: NodeIndex,
    send_routers: Vec<NodeIndex>,
    improve_value: f64,
    is_pair: bool,
}

impl ImpInfo {
    pub fn new(
        devices: Vec<Device>,
        receive_router: NodeIndex,
        send_routers: Vec<NodeIndex>,
        improve_value: f64,
        is_pair: bool,
    ) -> Self {
        Self {
            devices,
            receive_router,
            send_routers,
            improve_value,
            is_pair,
        }
    }
    pub fn devices(&self) -> &Vec<Device> {
        &self.devices
    }

    pub fn receive_router(&self) -> NodeIndex {
        self.receive_router
    }

    pub fn send_routers(&self) -> &Vec<NodeIndex> {
        &self.send_routers
    }

    pub fn improve_value(&self) -> f64 {
        self.improve_value
    }

    pub fn is_pair(&self) -> bool {
        self.is_pair
    }
}
