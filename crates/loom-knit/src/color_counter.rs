use std::collections::HashMap;
use crossterm::style::Color;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_color_counter() {
        let counter = ColorCounter {
            color_hashmap: HashMap::new(),
        };

        let queue = counter.get_shuffled_queue();
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_single_color_single_count() {
        let mut map = HashMap::new();
        map.insert(Color::Red, 1);

        let counter = ColorCounter {
            color_hashmap: map,
        };

        let queue = counter.get_shuffled_queue();
        assert_eq!(queue.len(), 1);
        assert_eq!(queue[0], Color::Red);
    }

    #[test]
    fn test_single_color_multiple_count() {
        let mut map = HashMap::new();
        map.insert(Color::Blue, 5);

        let counter = ColorCounter {
            color_hashmap: map,
        };

        let queue = counter.get_shuffled_queue();
        assert_eq!(queue.len(), 5);

        // All should be blue
        for color in queue {
            assert_eq!(color, Color::Blue);
        }
    }

    #[test]
    fn test_multiple_colors_correct_counts() {
        let mut map = HashMap::new();
        map.insert(Color::Red, 3);
        map.insert(Color::Blue, 2);
        map.insert(Color::Green, 4);

        let counter = ColorCounter {
            color_hashmap: map,
        };

        let queue = counter.get_shuffled_queue();
        assert_eq!(queue.len(), 9);

        // Count each color in the result
        let mut red_count = 0;
        let mut blue_count = 0;
        let mut green_count = 0;

        for color in queue {
            match color {
                Color::Red => red_count += 1,
                Color::Blue => blue_count += 1,
                Color::Green => green_count += 1,
                _ => panic!("Unexpected color"),
            }
        }

        assert_eq!(red_count, 3);
        assert_eq!(blue_count, 2);
        assert_eq!(green_count, 4);
    }

    #[test]
    fn test_queue_is_shuffled() {
        // Create a counter with enough elements to test shuffling
        let mut map = HashMap::new();
        map.insert(Color::Red, 10);
        map.insert(Color::Blue, 10);

        let counter = ColorCounter {
            color_hashmap: map,
        };

        let queue = counter.get_shuffled_queue();

        // It's extremely unlikely that a shuffled queue of 20 elements
        // would be perfectly sorted (all Reds then all Blues or vice versa)
        // This is a probabilistic test
        let mut is_perfectly_sorted = true;
        let first_color = queue[0];

        for (i, color) in queue.iter().enumerate() {
            if i < 10 && *color != first_color {
                is_perfectly_sorted = false;
                break;
            }
            if i >= 10 && *color == first_color {
                is_perfectly_sorted = false;
                break;
            }
        }

        // Very unlikely to be perfectly sorted after shuffle
        // (though theoretically possible, probability is ~1 in 184,756)
        assert!(!is_perfectly_sorted || queue.len() < 4);
    }
}
