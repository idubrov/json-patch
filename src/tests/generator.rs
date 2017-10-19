use serde_json::{Value, Map};
use super::rand;
use super::rand::Rng;

pub struct Params {
    pub array_size: usize,
    pub map_size: usize,
    pub value_size: usize,
    pub depth: usize,
    pub key_size: usize
}

impl Default for Params {
    fn default() -> Self {
        Params {
            array_size: 5,
            map_size: 5,
            value_size: 1000,
            depth: 10,
            key_size: 100
        }
    }
}

fn rand_str<R: Rng>(rng: &mut R, max_len: usize) -> String {
    let len = rng.gen::<usize>() % max_len + 1;
    rng.gen_ascii_chars().take(len).collect()
}

fn rand_literal<R: Rng>(rng: &mut R, value_size: usize) -> Value {
    match rng.gen::<u32>() % 4 {
        0 => Value::Null,
        1 => Value::String(rand_str(rng, value_size)),
        2 => Value::Bool(false),
        3 => Value::from(rng.gen::<u64>()),
        _ => panic!()
    }
}

impl Params {
    pub fn gen<R: Rng>(&self, rng: &mut R) -> Value {
        self.gen_internal(self.depth, rng)
    }

    fn gen_internal<R: Rng>(&self, depth: usize, rng: &mut R) -> Value {
        if depth == 0 {
            rand_literal(rng, self.value_size)
        } else if rng.gen::<bool>() {
            let vec: Vec<Value> = (0..=rng.gen::<usize>() % self.array_size)
                .map(|_| self.gen_internal(depth - 1, rng))
                .collect();
            Value::from(vec)
        } else {
            let map: Map<String, Value> = (0..=rng.gen::<usize>() % self.map_size)
                .map(|_| (rand_str(rng, self.key_size), self.gen_internal(depth - 1, rng)))
                .collect();
            Value::from(map)
        }
    }
}

fn gen_path_internal(value: &Value, rng: &mut rand::StdRng, path: &mut Vec<String>) {
    let mut cur = value;
    loop {
        match *cur {
            Value::Array(ref arr) => {
                let pos = rng.gen::<usize>() % arr.len();
                path.push(pos.to_string());
                cur = &arr[pos];
            },
            Value::Object(ref map) => {
                let pos = rng.gen::<usize>() % map.len();
                let (key, _) = map.iter().skip(pos).next().unwrap();
                path.push(key.clone());
                cur = &map[key];
            }
            _ => return
        }
    }
}

pub fn gen_path(value: &Value, rng: &mut rand::StdRng) -> String {
    let mut path = Vec::new();
    gen_path_internal(value, rng, &mut path);
    let take = rng.gen::<usize>() % (path.len() + 1);

    let mut buf = String::new();
    path.iter().take(take).for_each(|x| { buf.push('/'); buf += x; });
    buf
}