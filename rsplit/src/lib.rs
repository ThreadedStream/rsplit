use rand::{thread_rng, Rng, distributions::Uniform};
use rand::distributions::Alphanumeric;
use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver, RecvError};
use std::fmt;

// threshold value below which no thread spawning is possible
static MIN_BOUND: usize = 4;
static MAX_BOUND: usize = 1000;
static PART_FACTOR: usize = 300;
static FALLBACK: u32 = 16;


fn worker<T: 'static + std::marker::Send + Clone, R: 'static + std::marker::Send>(v: Vec<T>, mapper: fn (t: T) -> R) -> Vec<R> {
    if v.len() < MIN_BOUND {
        // do it serially
        let result = v.into_iter()
            .map(|x| { mapper(x) })
            .collect();

        return result;
    }

    // my processor can barely handle this amount of threads running at the same time
    if v.len() <= MAX_BOUND {

        let mut thread_pool = vec![];
        for chunk in v {
            thread_pool.push(thread::spawn(move || {
                return mapper(chunk)
            }));
        }

        let result = thread_pool.into_iter().map(|th| th.join().unwrap()).collect();
        return result;
    }   

    // a small benchmark, 
    // cw - computational work (basically, input length)
    // part_factor - partitioning factor, i.e number of elements processed by each thread
    // time - time taken to process the particular cw with a given part_factor 
    // part_factor = 100
    // cw - 2000, part_factor - 100, time - 7.672s
    // cw - 3000, part_factor - 100, time - 7.340s
    // cw - 4000, part_factor - 100, time - 18.660s 
    // The last sample surprised me quite a lot, so I decided to rerun it 
    // and ended up with a fair result of 13.057 seconds
    // Moreover, I rerun this sample several times and ,after a bit of oscillation
    // between 12-13 and 17-18, it has eventually stabilized to ~12s
    // ################################################
    // part_factor = 200
    // cw - 2000, part_factor - 200, time - 9.251s
    // cw - 3000, part_factor - 200, time - 12.902s
    // cw - 4000, part_factor - 200, time - 12.967s
    // ################################################
    // part_factor = 300
    // cw - 2000, part_factor - 300, time - 9.255s
    // cw - 3000, part_factor - 300, time - 13.445s
    // cw - 4000, part_factor - 300, time - 11.965s
    let mut thread_pool = vec![];
    let chunks: Vec<Vec<T>> = v.chunks(PART_FACTOR).map(|s| s.into()).collect();
    for chunk in chunks {
        thread_pool.push(thread::spawn(move || { 
            let vec: Vec<R> = chunk.into_iter()
                        .map(|x| mapper(x))
                        .collect();
            return vec;
        }))        
    }

    let processed_chunks: Vec<Vec<R>> = thread_pool.into_iter().map(|th| th.join().unwrap()).collect();
    
    let flattened_chunks: Vec<R> = processed_chunks.into_iter().flatten().collect();
    return flattened_chunks;
}



fn worker_channels<T: 'static + std::marker::Send, R: 'static + std::marker::Send>(v: Vec<T>, mapper: fn(t: T) -> R) -> Vec<R>{
    if v.len() < MIN_BOUND {
        // do it serially
        let result = v.into_iter()
            .map(|x| { mapper(x) })
            .collect();

        return result;
    }    

    let num_of_threads = v.len();
    let (tx, rx): (Sender<R>, Receiver<R>) = mpsc::channel();
    let mut children = vec![];

    for chunk in v {
        let thread_tx = tx.clone();
        let child = thread::spawn(move || {
            thread_tx.send(mapper(chunk)).unwrap();
        });
        children.push(child);
    }

    let mut tchunks = vec![];
    for _ in 0..num_of_threads {
        tchunks.push(rx.recv().unwrap());
    }    

    // waiting for all of the threads to complete their job
    for child in children {
        child.join();
    }

    return tchunks;
}

mod helpers {
    use super::*;
    use std::num;

    pub(crate) static MATCH: &str = "That";

    pub fn cmp_vec<T: PartialEq>(a: &Vec<T>, b: &Vec<T>) -> bool {
        let matching = a.iter().zip(b.iter()).filter(|&(a, b)| a == b).count();
        matching == a.len() && matching == b.len()
    }


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

    pub fn is_matched(s: String) -> u32{
        if s == MATCH {
            1
        } else {
            0
        }
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
        let v_result = worker(v, helpers::validate_hash);
        assert_eq!(v_result.len(), v_len);

        let n = 2000;
        let mut hashes = Vec::with_capacity(n);

        for i in 1..n+1 {
            hashes.push(helpers::random_hash(i));
        }
        let hashes_result = worker(hashes, helpers::validate_hash);
        assert_eq!(hashes_result.len(), n);
    }

    #[test]
    fn magnitudes() {
        let v1 = vec![
                      helpers::random_vector(4), 
                      helpers::random_vector(5), 
                      helpers::random_vector(6), 
                      helpers::random_vector(7), 
                      helpers::random_vector(8)
                     ];    

        let v1_1 = v1.clone();
        let v1_len = v1.len();
        let mags_1 = worker(v1, helpers::vec_mag);
        assert_eq!(mags_1.len(), v1_len);

        let v2 = vec![
                      vec![2.0, 4.0, 6.0], 
                      vec![4.0, 6.0, 8.0, 10.0],  
                      vec![5.5, 7.2, 9.1, 11.5, 12.4] 
                     ];

        let eps = 0.0001;
        let v2_len = v2.len();
        let mags_2 = worker(v2, helpers::vec_mag);
        assert!((mags_2[0] - 56.0).abs() <= f32::EPSILON);
        assert!((mags_2[1] - 216.0).abs() <= f32::EPSILON);
        assert_eq!(mags_2[2].trunc(), 450.0);
    }   

    #[test]
    fn word_counting() {
        let text = String::from("The borrow check is Rust's \"secret sauce\" – it is tasked with enforcing a number of properties:

                    That all variables are initialized before they are used.
                    That you can't move the same value twice.
                    That you can't move a value while it is borrowed.
                    That you can't access a place while it is mutably borrowed (except through the reference).
                    That you can't mutate a place while it is immutably borrowed.
                    etc
                    The borrow checker operates on the MIR. An older implementation operated on the HIR. Doing borrow checking on MIR has several advantages:

                    The MIR is far less complex than the HIR; the radical desugaring helps prevent bugs in the borrow checker. (If you're curious, you can see a list of bugs that the MIR-based borrow checker fixes here.)
                    Even more importantly, using the MIR enables \"non-lexical lifetimes\", which are regions derived from the control-flow graph.
                ");

        let tokens: Vec<String> = text.split_whitespace().map(|s| s.to_string()).collect();
        let tokens_1 = tokens.clone();
        let result = worker(tokens, helpers::is_matched);
        assert_eq!(result.iter().sum::<u32>(), 5);
        let result_1 = worker_channels(tokens_1, helpers::is_matched);
        assert_eq!(result_1.iter().sum::<u32>(), 5);
    }

}