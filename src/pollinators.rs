use super::structs::*;
use super::*;
//use rand_distr::Uniform;
use rand::prelude::ThreadRng;
use itertools::Itertools;

pub trait Pollinator {
    fn global(rng: &mut ThreadRng, g: &Graph, best: &Coloring, other: &Coloring, lambda: f32) -> Coloring;
    fn local(rng: &mut ThreadRng, g: &Graph, x: &Coloring, k: usize, lambda: f32) -> Coloring;
}

pub struct CM;

impl Pollinator for CM {
    fn global(rng: &mut ThreadRng, g: &Graph, best: &Coloring, other: &Coloring, lambda: f32) -> Coloring {
        levy_pop1(g, rng, best, other, lambda)
    }
    fn local(rng: &mut ThreadRng, g: &Graph, x: &Coloring, k: usize, lambda: f32) -> Coloring {
        change_multiple(rng, g, x, k, lambda)           // Seems best
    }
}

pub struct CMB;

impl Pollinator for CMB {
    fn global(rng: &mut ThreadRng, g: &Graph, best: &Coloring, other: &Coloring, lambda: f32) -> Coloring {
        levy_pop1(g, rng, best, other, lambda)
    }
    fn local(rng: &mut ThreadRng, g: &Graph, x: &Coloring, k: usize, lambda: f32) -> Coloring {
        change_multiple_best(rng, g, x, k, lambda)    // Seems pretty good
    }
}

pub struct CA;

impl Pollinator for CA {
    fn global(rng: &mut ThreadRng, g: &Graph, best: &Coloring, other: &Coloring, lambda: f32) -> Coloring {
        levy_pop1(g, rng, best, other, lambda)
    }
    fn local(rng: &mut ThreadRng, g: &Graph, x: &Coloring, k: usize, lambda: f32) -> Coloring {
        change_all_critical_opt(g, x, k)     // Seems pretty good
    }
}

pub struct CMW;

impl Pollinator for CMW {
    fn global(rng: &mut ThreadRng, g: &Graph, best: &Coloring, other: &Coloring, lambda: f32) -> Coloring {
        levy_pop1(g, rng, best, other, lambda)
    }
    fn local(rng: &mut ThreadRng, g: &Graph, x: &Coloring, k: usize, lambda: f32) -> Coloring {
        change_multiple_worst(rng, g, x, k, lambda)   // Seems ok
    }
}


/*
*   Generate a random number from a levy distribution
*/
fn levy(rng: &mut ThreadRng, c: f32) -> f32 {
    let n: f32 = rng.sample(StandardNormal);
    c / n.powf(2.0)
}

/*
*   Sample from levy until a value in the given range is found
*/
fn adjusted_levy(rng: &mut ThreadRng, limit: usize, c: f32) -> usize {
    loop {
        let sample = levy(rng, c).round() as usize;
        if sample < limit {
            return sample;
        }
    }
}

fn levy_pop1(g: &Graph, rng: &mut ThreadRng, best: &Coloring, other: &Coloring, lambda: f32) -> Coloring {
    let n = best.len();
    let mut offspring = other.clone();
    let cutoff = adjusted_levy(rng, n, lambda);
    let indices = index::sample(rng, n, cutoff).into_vec();
    for &i in &indices {
        offspring[i] = best[i];
    }
    
    offspring.update_multiple_conflicts(&indices, other, g);

    offspring
}

fn levy_circ(g: &Graph, rng: &mut ThreadRng, x: &Coloring, lambda: f32) -> Coloring {
    let n = x.len();
    let mut offspring = x.clone();
    let cutoff = adjusted_levy(rng, n, lambda);
    let indices = index::sample(rng, n, cutoff);
    for (i, j) in indices.iter().zip(indices.iter().skip(1)) {
        offspring[i] = offspring[j];
    }
    offspring.update_multiple_conflicts(&indices.into_vec(), x, g);
    offspring
}

//pub fn partial_swap(rng: &mut ThreadRng, g: &Graph, x: &Coloring) -> Coloring {
//    let n = x.len();
//    let generator = Uniform::new(0, n);
//    let mut new = x.clone();
//    let mut prev_fit = conflicts(g, x);
//    if let MinMax(left, right) = rng.sample_iter(generator).take(2).minmax() {
//        for i in 1..right-left {
//            new.solution[left+1..left+i].reverse();
//            let new_fit = conflicts(g, &new);
//            if new_fit < prev_fit {
//                prev_fit = new_fit;
//            } else {
//                new.solution[left+1..left+i].reverse();
//            }
//        }
//    }
//    new
//}

// needs full graph
fn change_worst(rng: &mut ThreadRng, g: &Graph, x: &Coloring, k: usize) -> Coloring {
    //let worst = (0..g.len()).max_by_key(|&from| g[from].iter().filter(|&&to| x.solution[from] == x.solution[to]).count()).unwrap();
    let worst = (0..g.len()).max_by_key(|&from| x.conflicts[from]).unwrap();

    let neighbor_colors = (1..=k).chain(g[worst].iter().map(|&neighbor| x[neighbor])).counts();
    let best_color = neighbor_colors.iter().min_by_key(|&(_, count)| count).unwrap().0;
    let mut new = x.clone();
    new[worst] = *best_color;

    // Efficiently calculate new conflicts
    new.update_conflicts(worst, x, g);
    new
}

// needs full graph
fn change_multiple_worst(rng: &mut ThreadRng, g: &Graph, x: &Coloring, k: usize, lambda: f32) -> Coloring {
    let mut new = x.clone();
    let worst = (0..g.len()).filter(|&i| x.conflicts[i] != 0).sorted_unstable_by_key(|&from| x.conflicts[from]).rev();
    let n = adjusted_levy(rng, x.len(), lambda);
    for i in worst.take(n) {
        let neighbor_colors = (1..=k).chain(g[i].iter().map(|&neighbor| x[neighbor])).counts();
        let best_color = neighbor_colors.iter().min_by_key(|&(_, count)| count).unwrap().0;
        new[i] = *best_color;
    }
    new
}

// needs full graph
fn change_multiple_best(rng: &mut ThreadRng, g: &Graph, x: &Coloring, k: usize, lambda: f32) -> Coloring {
    let mut new = x.clone();
    let best = (0..g.len()).filter(|&i| x.conflicts[i] != 0).sorted_unstable_by_key(|&i| x.conflicts[i]);
    let n = adjusted_levy(rng, x.len(), lambda);

    let indices: Vec<usize> = best.take(n).collect();

    for &i in &indices {
        let neighbor_colors = (1..=k).chain(g[i].iter().map(|&neighbor| new[neighbor])).counts();
        let best_color = neighbor_colors.iter().min_by_key(|&(_, count)| count).unwrap().0;
        new[i] = *best_color;
    }

    new.update_multiple_conflicts(&indices, x, g);

    new
}

// needs full graph
fn try_change_critical(rng: &mut ThreadRng, g: &Graph, x: &Coloring, k: usize) -> Coloring {
    let mut new = x.clone();

    let critical: Vec<usize> = (0..g.len()).filter(|&i| x.conflicts[i] != 0).collect();
    let generator = Uniform::new(0, critical.len());
    let i = rng.sample(generator);
    let i = critical[i];
    let neighbor_colors = (1..=k).chain(g[i].iter().map(|&neighbor| new[neighbor])).counts();
    let best_color = neighbor_colors.iter().min_by_key(|&(_, count)| count).unwrap().0;
    new[i] = *best_color;

    new.update_conflicts(i, x, g);

    new
}

// needs full graph
fn change_one(rng: &mut ThreadRng, g: &Graph, x: &Coloring, k: usize) -> Coloring {
    let mut new = x.clone();
    let critical = (0..g.len()).filter(|&i| x.conflicts[i] != 0);
    for i in critical {
        let neighbor_colors = (1..=k).chain(g[i].iter().map(|&neighbor| new[neighbor])).counts();
        let (&best_color, &conflicts) = neighbor_colors.iter().min_by_key(|&(_, count)| count).unwrap();
        if let Some((&color, _)) = neighbor_colors.iter().find(|&(_, &count)| count == 1) {
            new[i] = color;
            new.update_conflicts(i, x, g);
            break;
        }
    }
    new
}

// needs full graph
fn change_multiple(rng: &mut ThreadRng, g: &Graph, x: &Coloring, k: usize, lambda: f32) -> Coloring {
    let mut new = x.clone();
    //let best = (0..g.len()).map(|from| x.conflicts[from]).enumerate().filter(|&(_, conflicts)| conflicts != 0).sorted_unstable_by_key(|&(_, conflicts)| conflicts);
    let best = (0..g.len()).filter(|&i| x.conflicts[i] != 0);
    let n = adjusted_levy(rng, x.len(), lambda);

    let indices: Vec<usize> = best.take(n).collect();

    for &i in &indices {
        let neighbor_colors = (1..=k).chain(g[i].iter().map(|&neighbor| new[neighbor])).counts();
        let best_color = neighbor_colors.iter().min_by_key(|&(_, count)| count).unwrap().0;
        new[i] = *best_color;
    }

    new.update_multiple_conflicts(&indices, x, g);

    new
}

// needs full graph
fn change_all_critical(g: &Graph, x: &Coloring, k: usize) -> Coloring {
    let mut new = x.clone();
    let best: Vec<usize> = (0..g.len()).filter(|&i| x.conflicts[i] != 0).collect();
    for &i in &best {
        let neighbor_colors = (1..=k).chain(g[i].iter().map(|&neighbor| new[neighbor])).counts();
        let best_color = neighbor_colors.iter().min_by_key(|&(_, count)| count).unwrap().0;
        new[i] = *best_color;
    }

    new.update_multiple_conflicts(&best, x, g);

    new
}

// needs full graph
fn change_all_critical_opt(g: &Graph, x: &Coloring, k: usize) -> Coloring {
    let mut new = x.clone();
    let critical: Vec<usize> = (0..g.len()).filter(|&i| new.conflicts[i] != 0).collect();

    for &i in &critical {
        let neighbor_colors = (1..=k).chain(g[i].iter().map(|&neighbor| new[neighbor])).counts();
        let mut best_count =  usize::MAX;
        let mut best_color = 0;
        for (color, count) in neighbor_colors {
            if count == 1 {
                best_color = color;
                break;
            } else if count < best_count {
                best_count = count;
                best_color = color;
            }
        }
        new[i] = best_color;
    }

    new.update_multiple_conflicts(&critical, x, g);
    new
}

//pub struct Mox;
//
//impl Pollinator for Mox {
//    fn global(rng: &mut ThreadRng, _g: &Graph, best: &Coloring, other: &Coloring) -> Coloring {
//        let n = best.len();
//        let mut offspring = Vec::new();
//        let prob = 0.1;
//        for i in 0..n {
//            if rng.gen_bool(prob) {
//                offspring.push(best[i]);
//            } else {
//                offspring.push(other[i]);
//            }
//        }
//        offspring
//        //std::cmp::min_by_key(offspring1, offspring2, |x| conflicts(g, x))
//    }
//
//    fn local(rng: &mut ThreadRng, _g: &Graph, x: &Coloring, _k: usize) -> Coloring {
//        let n = x.len();
//        // let mut rng = thread_rng();
//        let mut offspring = x.clone();
//        let cutoff = adjusted_levy(rng, n, LAMBDA, MU);
//        let indices = index::sample(rng, n, cutoff);
//        for (i, j) in indices.iter().zip(indices.iter().skip(1)) {
//            offspring[i] = offspring[j];
//        }
//        offspring
//    }
//}

//pub struct MIS;
//
//impl Pollinator for MIS {
//    fn global(rng: &mut ThreadRng, g: &Graph, best: &Coloring, other: &Coloring, lambda: f32) -> Coloring {
//        let n = best.len();
//        //let parent1 = best.iter().enumerate().into_group_map_by(|(node, color)| color);
//        //let parent2 = other.iter().enumerate().into_group_map_by(|(node, color)| color);
//
//        let parent1 = best.solution.iter().enumerate().sorted_unstable_by_key(|&(_, color)| color).group_by(|&(_, color)| color);
//        let mut parent1 = parent1.into_iter().map(|group| group.1.collect::<Vec<(usize, &usize)>>()).peekable();
//
//        let parent2 = other.solution.iter().enumerate().sorted_unstable_by_key(|&(_, color)| color).group_by(|&(_, color)| color);
//        let mut parent2 = parent2.into_iter().map(|group| group.1.collect::<Vec<(usize, &usize)>>()).peekable();
//
//        let mut merge = Vec::new();
//         
//
//        let prob = 0.5;
//        while parent1.peek().is_some() && parent2.peek().is_some() {
//            if rng.gen_bool(prob) {
//                merge.append(&mut parent1.next().unwrap());
//            } else {
//                merge.append(&mut parent2.next().unwrap());
//            }
//        }
//        
//        for mut group in parent1 {
//            merge.append(&mut group);
//        }
//
//        for mut group in parent2 {
//            merge.append(&mut group);
//        }
//
//        let mut child1 = vec![0; n];
//        let mut child2 = vec![0; n];
//
//        for (node, &color) in merge {
//            if child1[node] == 0 {
//                child1[node] = color;
//            } else {
//                child2[node] = color;
//            }
//        }
//        
//        Coloring::from_vec(child1, g)
//    }
//
//    fn local(rng: &mut ThreadRng, g: &Graph, x: &Coloring, k: usize, lambda: f32) -> Coloring {
//        change_worst(rng, g, x, k)
//        //partial_swap(rng, g, x)
//        //levy_circ(rng, x)
//    }
//}
