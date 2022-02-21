use rand::{thread_rng, Rng, distributions::Uniform};
use rand::distributions::Alphanumeric;
use std::thread;


// threshold value below which no thread spawning is possible
const THRESHOLD: usize = 4;
const FALLBACK: u32 = 16;


fn worker<T: 'static + std::marker::Send, R: 'static + std::marker::Send>(v: Vec<T>, mapper: fn (t: T) -> R) -> Vec<R> {
    if v.len() < THRESHOLD {
        // do it serially
        let result = v.into_iter()
            .map(|x| { mapper(x) })
            .collect();

        return result;
    }

    let mut thread_pool = vec![];
    for chunk in v {
        thread_pool.push(thread::spawn(move || {
            return mapper(chunk)
        }));
    }

    let result = thread_pool.into_iter().map(|th| th.join().unwrap()).collect();
    return result;
}

// fn main() {
//     let v = vec![random_hash(), random_hash(), random_hash(), random_hash(), random_hash(), random_hash()];
//     let result = worker(v, validate_hash);
//     for r in result {
//         println!("{}", r);
//     }
// }

mod helpers {
    use super::*;
    use std::num;

    pub fn random_hash(hash_len: usize) -> String {
        return thread_rng()
            .sample_iter(&Alphanumeric)
            .take(hash_len)
            .map(char::from)
            .collect();
    }    

    fn compute_hash_value(hash: String) -> u32 {
        return hash
            .chars()
            .map(|c| match c.to_digit(16) {
                Some(val) => return val * 996,
                None => return FALLBACK,
            })
            .sum();
    }

    pub fn validate_hash(hash: String) -> bool {
        let val = compute_hash_value(hash);
        // dummy validation, admit it.
        return val % 3 == 0
    }

    pub fn random_vector(dim: usize) -> Vec<f32>{   
        let mut rng = thread_rng();
        let range = Uniform::new(0.0, 20.0);

        let vals: Vec<f32> = (0..dim).map(|_| rng.sample(&range)).collect();
        return vals;        
    }


    // vector's magnitude value
    // calculated as a dot product of v with itself
    pub fn vec_mag(v: Vec<f32>) -> f32 {
        v.into_iter()
         .map(|x| x * x)
         .sum()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::num;

    #[test]
    fn hashes() {
        let v = vec![helpers::random_hash(20), 
                     helpers::random_hash(25), 
                     helpers::random_hash(20),
                     helpers::random_hash(15), 
                     helpers::random_hash(15), 
                     helpers::random_hash(30)];
        let v_len = v.len();

        // first, ensure that length of the result vector matches that of initial one
        let result = worker(v, helpers::validate_hash);
        assert_eq!(result.len(), v_len);


    }

    #[test]
    fn magnitudes() {
        let v1 = vec![helpers::random_vector(4), 
                     helpers::random_vector(5), 
                     helpers::random_vector(6), 
                     helpers::random_vector(7), 
                     helpers::random_vector(8)];

        let v1_len = v1.len();
        let mags_1 = worker(v1, helpers::vec_mag);
        assert_eq!(mags_1.len(), v1_len);

        let v2 = vec![vec![2.0, 4.0, 6.0], // 
                     vec![4.0, 6.0, 8.0, 10.0],
                     vec![5.5, 7.2, 9.1, 11.5, 12.4]];

        let v2_len = v2.len();
        let mags_2 = worker(v2, helpers::vec_mag);
        assert!((mags_2[0] - 56.0).abs() <= f32::EPSILON);
        assert!((mags_2[1] - 216.0).abs() <= f32::EPSILON);
        assert!((mags_2[2] - 450.90999999999997).abs() <= f32::EPSILON);
    }   

}