use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Clone, Copy)]
struct Elem {
    value: i32,
    index: usize,
}

fn main() -> std::io::Result<()> {
    let file = File::open("./src/input.txt")?;
    let lines = BufReader::new(file).lines();
    let values: Vec<i32> = lines.map(|l| l.unwrap().parse::<i32>().unwrap()).collect();
    let values: Vec<Elem> = values
        .into_iter()
        .enumerate()
        .map(|(i, v)| Elem { value: v, index: i })
        .collect();
    let length = values.len();
    let mut result = values.clone();
    for original in values.iter() {
        let current_index = result
            .iter()
            .position(|e| e.index == original.index)
            .unwrap();
        let mut new_index =
            ((current_index as i32 + original.value).rem_euclid((length - 1) as i32)) as usize;
        if new_index == 0 && original.value != 0 {
            new_index = length - 1;
        }
        let mut next_result = Vec::with_capacity(length);
        if new_index > current_index {
            next_result.extend_from_slice(&result[0..current_index]);
            next_result.extend_from_slice(&result[current_index + 1..new_index + 1]);
            next_result.push(*original);
            if new_index < length - 1 {
                next_result.extend_from_slice(&result[new_index + 1..length])
            }
        } else {
            next_result.extend_from_slice(&result[0..new_index]);
            next_result.push(*original);
            next_result.extend_from_slice(&result[new_index..current_index]);
            if current_index < length - 1 {
                next_result.extend_from_slice(&result[current_index + 1..length])
            }
        }
        result = next_result;
    }
    let message_start = result.iter().position(|e| e.value == 0).unwrap();
    let x = result[(message_start + 1000) % length].value;
    let y = result[(message_start + 2000) % length].value;
    let z = result[(message_start + 3000) % length].value;
    println!("x={}, y={}, z={}, sum={}", x, y, z, x + y + z);
    Ok(())
}
