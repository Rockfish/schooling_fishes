//------------------------------------------------------------------------
//
//  Name: Smoother.h
//
//  Desc: Template class to help calculate the average value of a history
//        of values. This can only be used with types that have a 'zero'
//        value and that have the += and / operators overloaded.
//
//        Example: Used to smooth frame rate calculations.
//
//  Author: Mat Buckland (fup@ai-junkie.com)
//
//------------------------------------------------------------------------

use std::ops::{AddAssign, Div};

#[derive(Debug)]
pub struct Smoother<T> {
    //this holds the history
    m_History: Vec<T>,

    m_iNextUpdateSlot: usize,

    // An example of the 'zero' value of the type to be smoothed.
    // This would be something like Vector2D(0,0)
    m_ZeroValue: T,
}

impl<T: Clone + AddAssign + Div<f32, Output = T>> Smoother<T> {
    //to instantiate a Smoother pass it the number of samples you want
    //to use in the smoothing, and an example of a 'zero' type
    pub fn new(sample_size: i32, zero_value: T) -> Self {
        let mut history = Vec::with_capacity(sample_size as usize);
        for _i in 0..sample_size {
            history.push(zero_value.clone());
        }

        Smoother {
            m_History: history,
            m_iNextUpdateSlot: 0,
            m_ZeroValue: zero_value,
        }
    }

    //each time you want to get a new average, feed it the most recent value
    //and this method will return an average over the last SampleSize updates
    pub fn update(&mut self, most_recent_value: T) -> T {
        //overwrite the oldest value with the newest
        self.m_History[self.m_iNextUpdateSlot] = most_recent_value;

        self.m_iNextUpdateSlot += 1;

        //make sure m_iNextUpdateSlot wraps around.
        if self.m_iNextUpdateSlot == self.m_History.len() {
            self.m_iNextUpdateSlot = 0;
        }

        //now to calculate the average of the history list
        let mut sum = self.m_ZeroValue.clone();

        for it in &self.m_History {
            sum += it.clone();
        }

        sum / (self.m_History.len() as f32)
    }
}
