use crate::filter::*;
use crate::delay::DelayLine;

use num_traits::{Float, FromPrimitive};


#[derive(Debug, Copy, Clone)]
pub struct Model<F: Float + Default, ModelFilter: Filter<F>>{
    pub filter: ModelFilter,
    pub delay: DelayLine<F>
}

impl<F: Float + Default + FromPrimitive, ModelFilter: Filter<F> + Default>Default for Model<F, ModelFilter>{
    fn default() -> Self {
        let f = ModelFilter::default();
        Self{
            filter: f,
            delay: DelayLine::new()
        }
    }
}

impl<F: Float + Default + FromPrimitive, ModelFilter: Filter<F>> Model<F, ModelFilter>{
    pub fn new(f: ModelFilter) -> Self{
        Self{
            filter: f,
            delay: DelayLine::new()
        }
    }
    
    pub fn process(&mut self, input: F) -> F{
        let delay_out = self.delay.pop();
        let f_result = self.filter.tick(delay_out);
                
        let delay_result = f_result + input;
        
        // Added constant decay parameter to prevent ringing
        self.delay.push(F::from_f64(0.98).unwrap() * delay_result);
        delay_out
    }
}