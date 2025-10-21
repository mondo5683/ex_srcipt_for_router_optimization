use crate::device::Device;
use clustering;
impl clustering::Elem for Device {
    fn dimensions(&self) -> usize {
        1
    }

    fn at(&self, _i: usize) -> f64 {
        self.get_learning_time()
    }
}
