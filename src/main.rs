#![warn(non_snake_case)]
#![allow(dead_code)]
#![allow(unused_variables)]

mod structs;
mod greedy;
mod pollinators;
mod bench;

use std::time::Instant;

use rand::{Rng, thread_rng};
use rand::seq::{index};
use rand_distr::{StandardNormal, Uniform};
use rayon::prelude::*;

use self::structs::*;
use self::greedy::*;
use self::pollinators::*;
use self::bench::*;

const MAX_GEN: usize = 50000;
const POP_SIZE: usize = 20;

fn discrete_fpa<T: Pollinator>(g: &Graph, n: usize, k: usize, options: Option<Parameters>, stop: Option<usize>) -> usize {
    //println!("Evaluating k = {}.", k);
    if let Some(limit) = stop {
        if k < limit {
            return limit;
        }
    }
    let Parameters { lambda, switch_p, lifetime_limit } = match options {
        Some(opt) => opt,
        None => Parameters::standard()
    };
    let mut pop = g.populate(n, k);
    for _ in 0..MAX_GEN {
        let best = pop.par_iter().min_by_key(|x| x.tot_conflicts).unwrap().clone();
        
        if best.tot_conflicts == 0 {
            //println!("\tFound solution at iteration {}.", it);
            return discrete_fpa::<T>(g, n, k-1, options, stop);
        }

        // Iterate through all solutions (in parallel)
        pop.par_iter_mut().for_each_init(|| thread_rng(), |rng, x| {
            let p = rng.gen_bool(switch_p);

            if x.lifetime >= lifetime_limit && p && *x != best {
                *x = Coloring::new(x.index, g, k, rng);
                x.lifetime = 0;
                return;
            }
            
            let x_new = if p && *x != best {
                // Biotic pollination
                T::global(rng, g, &best, x, lambda)
            } else {
                // Abiotic pollination
                T::local(rng, g, x, k, lambda)
            };
            
            if x_new.tot_conflicts <= x.tot_conflicts {
                if x_new.tot_conflicts == x.tot_conflicts {
                    x.lifetime += 1;
                } else {
                    x.lifetime = 0;
                }
                *x = x_new;
            } else {
                x.lifetime += 1;
            }
        });
    }
    k + 1
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args[1] == "bench" {
        if args.len() >= 3 {
            let from = args[2].parse().unwrap();
            let to = if args.len() == 4 {
                args[3].parse::<usize>().unwrap() + 1
            } else {
                from + 1
            };
            benchmark::<CA>(None, Some(from..to));
        } else {
            benchmark::<CA>(None, None);
        }
        return;
    } else if args[1] == "bench_param" {
        return bench_parameters();
    } else if args[1] == "bench_pol" {
        return bench_pollinators();
    } else if args.len() > 4 {
        panic!("Unsupported argument!");
    }
    let path = format!("graphs/{}.col", args[1]);
    let graph = Graph::read(&path);
    let max_d = graph.max_degree();
    //println!("Graph: {:?}", graph);
    println!("Num nodes: {}", graph.len());
    println!("Max degree: {}", max_d);


    let now = Instant::now();
    let (greedy, _greedy_sol) = dsatur2(&graph);
    let num_colors = discrete_fpa::<CA>(&graph, POP_SIZE, greedy-1, None, None);
    let elapsed_time = now.elapsed();

    println!("Num colors: {}", num_colors);
    println!("Running DFPA took {} seconds.", elapsed_time.as_secs());
}
