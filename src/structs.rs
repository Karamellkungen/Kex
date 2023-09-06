use std::ops::{Index, IndexMut};
use std::fs;
use std::collections::{HashSet, HashMap};
use rand::{Rng, thread_rng};
use rand::distributions::Uniform;

use rand::prelude::ThreadRng;
use itertools::Itertools;

//---------------------------------------------------------------------------------------//

#[derive(Clone, Copy)]
pub struct Parameters {
    pub lambda: f32,
    pub switch_p: f64,
    pub lifetime_limit: usize
}

impl Parameters {
    pub fn standard() -> Parameters {
        Parameters {
            lambda: 1.5,
            switch_p: 0.2,
            lifetime_limit: 30
        }
    }
}

//---------------------------------------------------------------------------------------//

#[derive(Clone, Debug)]
pub struct Coloring {
    pub index: usize,
    pub solution: Vec<usize>,
    pub conflicts: Vec<usize>,
    pub tot_conflicts: usize,
    pub lifetime: usize
}

impl Coloring {
    /*
    *   Generate a new coloring with k colors
    */
    pub fn new(index: usize, g: &Graph, k: usize, rng: &mut ThreadRng) -> Coloring {
        let n = g.len();
        let generator = Uniform::new_inclusive(1, k);
        let solution: Vec<usize> = rng.sample_iter(generator).take(n).collect();
        let conflicts: Vec<usize> = (0..n).map(|from| g[from].iter().filter(|&&to| solution[from] == solution[to]).count()).collect();
        let tot_conflicts = conflicts.iter().sum::<usize>() / 2;
        Coloring {index, solution, conflicts, tot_conflicts, lifetime: 0}
    }

    /*
    *   Initialize a coloring from a given solution vector
    */
    pub fn from_vec(index: usize, solution: Vec<usize>, g: &Graph) -> Coloring {
        let n = solution.len();
        let conflicts: Vec<usize> = (0..n).map(|from| g[from].iter().filter(|&&to| solution[from] == solution[to]).count()).collect();
        let tot_conflicts = conflicts.iter().sum::<usize>() / 2;
        Coloring {index, solution, conflicts, tot_conflicts, lifetime: 0}
    }

    /*
    *   Generate an empty solution
    */
    pub fn empty(index: usize, n: usize) -> Coloring {
        Coloring {index, solution: vec![0; n], conflicts: vec![0; n], tot_conflicts: 0, lifetime: 0}
    }

    /*
    *   Return the length of the coloring
    */
    pub fn len(&self) -> usize {
        self.solution.len()
    }
    //   3
    // 3 - 3
    // Local pollination:
    //   3
    // 2 - 6
    

    /*
    *   Efficiently update the conflicts when modified from a previous state.
    *   Also refreshes the total number of conflicts.
    *   This (probably) only works when modifying a single index.
    */
    pub fn update_conflicts(&mut self, modified_index: usize, prev: &Coloring, g: &Graph) {
        self.conflicts[modified_index] = 0;
        for &neighbor in g[modified_index].iter() {
            if self[neighbor] == self[modified_index] {
                self.conflicts[modified_index] += 1;
                if prev[neighbor] != prev[modified_index] {
                    self.conflicts[neighbor] += 1;
                }
            } else if prev[neighbor] == prev[modified_index] {
                self.conflicts[neighbor] -= 1;
            }
        }
        self.refresh_tot_conflicts();
    }

    /*
    *   Update conflicts based on multiple changes. This method also refreshes the colorings total number of conflicts
    *   This (probably) requires a full graph
    */
    pub fn update_multiple_conflicts(&mut self, indices: &Vec<usize>, prev: &Coloring, g: &Graph) {
        let mut counter: HashMap<usize, usize> = HashMap::new();
        for &i in indices {
            let conflicts_i = counter.entry(i).or_default();
            for &neighbor in &g[i] {
                if self[i] == self[neighbor] {
                    *conflicts_i += 1;
                    if prev[i] != prev[neighbor] {
                        self.conflicts[neighbor] += 1;
                    }
                } else if self[i] != self[neighbor] && prev[i] == prev[neighbor] {
                    self.conflicts[neighbor] -= 1;
                }
            }
        }
        for (i, conflicts) in counter.into_iter() {
            self.conflicts[i] = conflicts;
        }
        self.refresh_tot_conflicts();
    }

    /*
    *   Re-calculate the colorings total number of conflicts
    */
    pub fn refresh_tot_conflicts(&mut self) {
        self.tot_conflicts = self.conflicts.iter().sum::<usize>() / 2;
    }

    /*
    *   Calculate the number of used colors
    */
    pub fn get_k(&self) -> usize {
        self.solution.iter().unique().count()
    }
}

impl Index<usize> for Coloring {
    type Output = usize;
    
    fn index(&self, i: usize) -> &Self::Output {
        &self.solution[i]
    }
}

impl IndexMut<usize> for Coloring {
    
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.solution[i]
    }
}

impl PartialEq for Coloring {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl Eq for Coloring {}

//---------------------------------------------------------------------------------------//

#[derive(Debug)]
pub struct Graph {
    content: Vec<Vec<usize>>
}

impl Graph {
    /*
    * Read full graph from file
    */
    pub fn read(file_name: &str) -> Graph {
        Graph {content: read_graph(file_name, false).unwrap()}
    }

    /*
    *   Read simple graph from file
    */
    pub fn read_simple(file_name: &str) -> Graph {
        Graph {content: read_graph(file_name, true).unwrap()}
    }

    pub fn len(&self) -> usize {
        self.content.len()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Vec<usize>> {
        self.content.iter()
    }

    /*
    *   Calculate the maximum degree opf a given graph
    */
    pub fn max_degree(&self) -> usize {
        self.iter().map(|vec| vec.len()).max().unwrap()
    }

    /*
    *   Initialize a population with random coloringsw
    */
    pub fn populate(&self, n: usize, k: usize) -> Vec<Coloring> {
        let mut rng = thread_rng();
        let mut pop = Vec::with_capacity(n);
        for i in 0..n {
            pop.push(Coloring::new(i, &self, k, &mut rng));
        }
        pop
    }
}

impl Index<usize> for Graph {
    type Output = Vec<usize>;

    fn index(&self, index: usize) -> &Vec<usize> {
        &self.content[index]
    }
}

/*
*   Read and parse a graph from the specified file
*/
fn read_graph(file_name: &str, simple: bool) -> Option<Vec<Vec<usize>>> {
    let content = fs::read_to_string(file_name).ok()?;
    let mut lines = content.split_terminator("\n")
        .filter(|&line| line != "" && &line[0..1] != "c")
        .map(|line| line.split_whitespace().collect::<Vec<&str>>());

    let header = lines.next()?;
    let num_nodes: usize = header[2].parse().ok()?;

    let mut graph = vec![HashSet::new(); num_nodes];

    for line in lines {
        let from = line[1].parse::<usize>().ok()? - 1;
        let to = line[2].parse::<usize>().ok()? - 1;
        if simple {
            if !graph[to].contains(&from) {
                graph[from].insert(to);
            }
        } else {
            graph[from].insert(to);
            graph[to].insert(from);
        }
        
    }

    Some(graph.iter_mut().map(|set| set.drain().collect()).collect())
}