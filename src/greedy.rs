use super::structs::*;

use std::collections::{HashSet};
use itertools::*;

pub fn dsatur(g: &Graph) -> (usize, Coloring) {
    let n = g.len();
    let mut c = Coloring::empty(0, n);
    for _ in 0..n {
        let mut indexes = (0..n).filter(|&i| c[i] == 0);
        let first = indexes.next().unwrap();
        let mut candidates = (Vec::from([first]), saturation(g, &c, first));
        for i in indexes {
            let sat = saturation(g, &c, i);
            if sat > candidates.1 {
                candidates = (Vec::from([i]), sat);
            } else if sat == candidates.1 {
                candidates.0.push(i);
            }
        }

        let v = if candidates.0.len() == 1 {
            candidates.0[0]
        } else {
            tiebreaker(g, &c, candidates.0)
        };

        let neighbor_colors: HashSet<usize> = g[v].iter().map(|&neighbor| c[neighbor]).collect();

        let min_color = (1..).find(|color| !neighbor_colors.contains(color)).unwrap();
        
        c[v] = min_color;
    }

    (c.get_k() ,c)
}

pub fn dsatur2(g: &Graph) -> (usize, Coloring) {
    let n = g.len();
    let mut c = Coloring::empty(0, n);
    let mut saturations = vec![0; g.len()];
    for _ in 0..n {
        let mut indexes = (0..n).filter(|&i| c[i] == 0);
        let first = indexes.next().unwrap();
        let mut candidates = (Vec::from([first]), saturations[first]);
        for i in indexes {
            let sat = saturations[i];
            if sat > candidates.1 {
                candidates = (Vec::from([i]), sat);
            } else if sat == candidates.1 {
                candidates.0.push(i);
            }
        }

        let v = if candidates.0.len() == 1 {
            candidates.0[0]
        } else {
            tiebreaker(g, &c, candidates.0)
        };

        let neighbor_colors: HashSet<usize> = g[v].iter().map(|&neighbor| c[neighbor]).collect();

        let min_color = (1..).find(|color| !neighbor_colors.contains(color)).unwrap();
        
        c[v] = min_color;

        for &i in &g[v] {
            saturations[i] = saturation(g, &c, i);
        }
    }

    (c.get_k(), c)
}

pub fn greedy(g: &Graph) -> (usize, Coloring) {
    let mut c = vec![0; g.len()];
    for i in 0..g.len() {
        let neighbor_colors: HashSet<usize> = g[i].iter().map(|&neighbor| c[neighbor]).collect();
        let min_color = (1..).find(|color| !neighbor_colors.contains(color)).unwrap();
        c[i] = min_color;
    }

    let k = c.iter().unique().count();

    (k, Coloring::from_vec(0, c, g))
}

fn saturation(g: &Graph, c: &Coloring, i: usize) -> usize {
    let neighbors = &g[i];
    neighbors.iter().map(|&j| c[j]).unique().count()
}

fn tiebreaker(g: &Graph, c: &Coloring, indexes: Vec<usize>) -> usize {
    *indexes.iter()
        .max_by_key(|&&i| 
            g[i].iter().filter(|&&neighbor| c[neighbor] == 0).count())
        .unwrap()
}