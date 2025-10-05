use std::collections::HashMap;
use crossterm::style::{
    Color,
    Stylize
};

use std::fmt;

use rand::prelude::*;
use rand::seq::SliceRandom;

pub struct ColorCounter{
    pub color_hashmap: HashMap<Color, u16>
}

impl ColorCounter {
    pub fn get_shuffled_queue(self: &Self) -> Vec<Color>{
        let mut rng = rand::rng();
        let mut result: Vec<Color> = Vec::new();
        for (key, value) in &self.color_hashmap{
            for _ in 0..*value{
                result.push(key.clone())
            } 
        }
        result.shuffle(&mut rng);
        return result;
    }

}
