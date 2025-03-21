pub struct LFO{
    x: f32,
    rate: f32,
    pub amount: f32
}

impl Default for LFO{
    fn default() -> Self {
        Self{
            x: 0.0,
            rate: 0.0,
            amount: 0.0
        }
    }
}

impl LFO{
    pub fn next(&mut self) -> f32{
        let result = self.amount * f32::sin(2.0 * 3.141519 * self.x / 441000.0);
        self.x += self.rate;
        result
    }
    pub fn set_freq(&mut self, f: f32){
        self.rate = f;
    }    
}