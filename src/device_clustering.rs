use clustering;
use crate::device::Device;
impl clustering::Elem for Device{


    fn dimensions(&self) -> usize{
        1
    }

    fn at(&self, i:usize) -> f64{
       self.get_learning_time() 
    }
}
