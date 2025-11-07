#[derive(Clone)]
pub struct QueueData {
    buyer_improved_value: f64,
    seller_improved_value: f64,
    improve_value: f64,
    add_assume_variance: f64,
    is_add_assume_variance_calculated: bool,
}
impl QueueData {
    pub fn new(
        buyer_improved_value: f64,
        seller_improved_value: f64,
        improve_value: f64,
        add_assume_variance: f64,
    ) -> Self {
        Self {
            buyer_improved_value,
            seller_improved_value,
            improve_value,
            add_assume_variance,
            is_add_assume_variance_calculated: false,
        }
    }
    pub fn get_buyer_improved_value(&self) -> f64 {
        self.buyer_improved_value
    }
    pub fn set_buyer_improved_value(&mut self, value: f64) {
        self.buyer_improved_value = value;
    }
    pub fn get_seller_improved_value(&self) -> f64 {
        self.seller_improved_value
    }
    pub fn set_seller_improved_value(&mut self, value: f64) {
        self.seller_improved_value = value;
    }
    pub fn get_improve_value(&self) -> f64 {
        self.improve_value
    }
    pub fn set_improve_value(&mut self, value: f64) {
        self.improve_value = value;
    }
    pub fn get_assume_add_variance(&self) -> f64 {
        self.add_assume_variance
    }
    pub fn set_assume_add_variance(&mut self, value: f64) {
        self.add_assume_variance = value;
    }
    pub fn is_variance_checked(&self) -> bool {
        self.is_add_assume_variance_calculated
    }
    pub fn set_variance_checked(&mut self, b: bool) {
        self.is_add_assume_variance_calculated = b;
    }
}
