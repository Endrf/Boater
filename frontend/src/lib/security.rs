use rand::distributions::Alphanumeric;
use rand::thread_rng;
use rand::Rng;

pub fn code_generator(length: usize) -> String { thread_rng().sample_iter(&Alphanumeric).take(length).map(char::from).collect() }