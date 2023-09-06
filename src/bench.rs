use std::{time::Instant, ops::Range};
use prettytable::*;
use std::fs::OpenOptions;

use crate::*;


static _TESTS_SIMPLE: &'static [(&str, usize)] = &[
    ("myciel3", 4),
    ("myciel4", 5),
    ("queen5_5", 5),
    ("queen6_6", 6),
    ("queen7_7", 7),
    ("myciel5", 6),
    ("huck", 11),
    ("jean", 10),
    ("david", 11),
    ("myciel6", 7),
    ("games120", 9),
    ("miles500", 20),
    ("miles250", 8),
    ("anna", 11)
];

static _TESTS_: &'static [(&str, usize)] = &[
    ("r250.5", 65)
];

static _TESTS: &'static [(&str, usize)] = &[
    ("queen7_7", 7),
    ("DSJC125.1", 5),
    ("DSJC125.9", 44),
    ("queen8_8", 9),
    ("queen9_9", 10)
];

static FINAL_TESTS: &'static [(&str, usize)] = &[
    ("DSJC125.1", 5), 
    ("DSJC125.5", 17), 
    ("DSJC125.9", 44), 
    ("le450_25a", 25),
    ("le450_25b", 25),
    ("le450_5a", 5),
    ("le450_5b", 5),
    ("le450_5c", 5),
    ("flat300_28_0", 28),
    ("r250.5", 65)
];

pub fn bench_pollinators() {

    let mut table = table!(["Pollinator", "Num_colors", "CPU_time"]);

    bench_single_pol::<CM>(&mut table, "CM");

    bench_single_pol::<CMB>(&mut table, "CMB");

    bench_single_pol::<CMW>(&mut table, "CMW");

    bench_single_pol::<CA>(&mut table, "CA");

    let mut file = std::fs::File::create("out/bench_pollinators.col").expect("");
    table.print(&mut file).expect("");
}

fn bench_single_pol<T: Pollinator>(table: &mut prettytable::Table, name: &str) {
    println!("Evaluating {}:", name);
    let now = Instant::now();
    let num_colors = benchmark::<T>(None, None);
    let elapsed_time = now.elapsed().as_millis();
    table.add_row(row![name, num_colors, elapsed_time]);
    println!("");
}

pub fn bench_parameters() {
    let lambdas = [1.0, 1.25, 1.5, 1.75];
    let switches = [0.2, 0.4, 0.6, 0.8];
    //let lifetimes = [10, 30, 50, 70];

    let mut options = Parameters::standard();

    let mut table =  table!(["lambda", "switch_p", "num_colors","CPU_time"]);

    //let mut best = (0.0, 0.0);
    //let mut best_coloring = f32::INFINITY;

    for lambda in lambdas {
        for switch_p in switches {
            println!("Evaluating lambda {}, switch_p {}", lambda, switch_p);
            options.lambda = lambda;
            options.switch_p = switch_p;
            let now = Instant::now();
            let num_colors = benchmark::<CA>(Some(options), None);
            let elapsed_time = now.elapsed().as_millis();
            table.add_row(row![lambda, switch_p, num_colors, elapsed_time]);
            //if num_colors < best_coloring {
            //    best_coloring = num_colors;
            //    best = (lambda, switch_p);
            //}
            println!("");
        }
    }
    let mut file = std::fs::File::create("out/CA/bench_lambda-switch.col").expect("");
    table.print(&mut file).expect("");

    //options.lambda = best.0;
    //options.switch_p = best.1;
//
    //println!("");

    //let mut table =  table!(["lifetimes", "num_colors","CPU_time"]);
//
    //for lifetime_limit in lifetimes {
    //    println!("Evaluating lifetime_limit: {}", lifetime_limit);
    //    options.lifetime_limit = lifetime_limit;
    //    let now = Instant::now();
    //    let num_colors = benchmark::<CM>(Some(options));
    //    let elapsed_time = now.elapsed().as_millis();
    //    table.add_row(row![lifetime_limit, num_colors, elapsed_time]);
    //}
    //let mut file = std::fs::File::create("out/bench_lifetimes.col").expect("");
    //table.print(&mut file).expect("");

}

pub fn benchmark<T: Pollinator>(options: Option<Parameters>, range: Option<Range<usize>>) -> f32 {
    let mut tot_colors = 0.0;
    let mut table = table!(["Graph", "k*", "k_init", "DFPA", "Best", "Average"]);

    let graphs = match range {
        Some(range) => FINAL_TESTS[range].iter(),
        None => FINAL_TESTS.iter()
    };

    for &(filename, chrom) in graphs {
        const NUM_TRIES: usize = 10;
        let path = format!("graphs/{}.col", filename);
        let graph = Graph::read(&path);
        //let max_d = graph.max_degree();
        //println!("Evaluating {}", filename);
        //println!("Num nodes: {}", graph.len());
        //println!("Max degree: {}", max_d);

        let mut avg_color = 0;
        
        let (greedy, _) = dsatur2(&graph);

        let mut tries = Vec::new();

        let now = Instant::now();
        for it in 0..NUM_TRIES {
            println!("Iteration: {}/{}", it+1, NUM_TRIES);
            //let num_colors = discrete_fpa::<T>(&graph, POP_SIZE, max_d, options, Some(chrom));
            let num_colors = discrete_fpa::<T>(&graph, POP_SIZE, greedy-1, options, Some(chrom));
            avg_color += num_colors;
            tries.push(num_colors);
        }
        let elapsed_time = now.elapsed();
        let avg_time = elapsed_time.as_millis() as f32 / NUM_TRIES as f32;
        let avg_color = avg_color as f32 / NUM_TRIES as f32;
        tot_colors += avg_color;
        println!("Graph: {}, Chromatic number: {}, Average number of colors: {}, Greedy solution: {}, Average execution time: {}", filename, chrom, avg_color, greedy, avg_time);
        table.add_row(row![filename, chrom, greedy-1, format!("{:?}", tries), tries.iter().min().unwrap(), avg_color,]);
    }
    //let mut file = std::fs::File::create("out/final.col").expect(""); // Create new file
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open("out/final.col")
        .unwrap();
    
    table.print(&mut file).expect("");
    
    tot_colors
}