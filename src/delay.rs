use num_traits::{Float, FromPrimitive};

const MAX_LENGTH: usize = 5000;

#[derive(Debug, Copy, Clone)]
pub struct DelayLine<T>{
    pub amount: usize,
    buffer: [T; MAX_LENGTH],
    wp: usize,
    y0: T
}

fn wrap_value(v: isize) -> usize{
    if v < 0{
        ((v + MAX_LENGTH as isize) as usize) % MAX_LENGTH
    }
    else{
        (v as usize) % MAX_LENGTH
    }
}

impl<'a, T: std::marker::Copy + Float + Default + FromPrimitive + 'a> DelayLine<T>{
    pub fn new() -> Self{
        Self{
            amount: 0,
            buffer: [T::default(); MAX_LENGTH],
            wp: 0,
            y0: T::default()
        }
    }
    pub fn push(&mut self, data: T){
        self.buffer[self.wp] = data;
        self.wp = (self.wp + 1) % MAX_LENGTH;
    }
    pub fn pop(&mut self) -> T{
        let rp: isize = (self.wp as isize) - (self.amount as isize);
        self.y0 = self.buffer[wrap_value(rp)];
        self.y0
    }
    pub fn set_frequency(&mut self, f: f32, sampling: f32){
        let delay = sampling / f;
        self.amount = delay.round() as usize;
        if self.amount >= MAX_LENGTH{
            self.amount = MAX_LENGTH - 1;
        }
    }
}