use rand::{rngs::ThreadRng, Rng, thread_rng};

/*
#######################################
###################^###################
################/     \################
#############/     @     \#############
##########/                 \##########
#######/                       \#######
####/              @              \####
####|  \                       /  |####
####|     \                 /     |####
####|  @     \     @     /        |####
####|           \     /           |####
####|          @   v              |####
####|      @       |       @      |####
####|_ @           |             _|####
#######\_          |          _/#######
##########\_   @   |       _/##########
#############\_    |    _/#############
################\_ | _/################
#######################################
 */
/// Represents an entry for WeightedRandom
#[derive(Clone)]
struct Entry<T: Clone> {
    accumulated_weight: f64,
    value: T
}

/// Acts as a bag to pull random entries from
/// # Example
/// ```
/// let mut rand_bag: WeightedRandom<i32> = WeightedRandomBuilder::new()
///     .add_entry(0, 0.5)
///     .add_entry(1, 0.5)
///     .finalize();
/// 
/// match rand_bag.get_rand() {
///     Some(i) => println!("{}", i),
///     None => println!("no worky")
/// };
/// ```
pub struct WeightedRandom<T: Clone> {
    entries: Vec<Entry<T>>,
    accumulated_weight: f64,
    rng: ThreadRng,
}

impl <T: Clone>  WeightedRandom<T> {
    pub fn get_rand(&mut self) -> Option<T> {
        let rand: f64 = self.rng.gen_range(0.0..self.accumulated_weight);

        for entry in &self.entries {
            if entry.accumulated_weight >= rand {
                return Option::Some(entry.value.clone());
            }
        }
        Option::None
    }
}

pub struct WeightedRandomBuilder<T: Clone> {
    entries: Vec<Entry<T>>,
    accumulated_weight: f64,
    rng: ThreadRng,
}

impl <T: Clone> WeightedRandomBuilder<T> {
    pub fn new() -> WeightedRandomBuilder<T> {
        WeightedRandomBuilder {
            entries: vec![],
            accumulated_weight: 0f64,
            rng: thread_rng(),
        }
    }

    pub fn add_entry(&mut self, value: T, weight: f64) -> &mut WeightedRandomBuilder<T>{
        self.accumulated_weight += weight;
        self.entries.push(Entry { accumulated_weight: self.accumulated_weight, value: value });
        self
    }

    pub fn finalize(&self) -> WeightedRandom<T>{
        WeightedRandom { entries: self.entries.clone(), accumulated_weight: self.accumulated_weight, rng: self.rng.clone() }
    }
}