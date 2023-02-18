use std::collections::HashMap;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::{Rng, SeedableRng};
use termgraph::DirectedGraph;

pub fn random_render(c: &mut Criterion) {
    c.bench_function("[random] render 100 Nodes - 120 Edges", |b| {
        let mut rng = rand::rngs::SmallRng::from_seed([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);

        let graph = {
            let mut tmp = DirectedGraph::new();

            tmp.add_nodes((0..100).map(|idx| (idx, format!("test-{idx}"))));
            tmp.add_edges(
                (0..120)
                    .map(|_| {
                        let src = rng.gen_range(0..100);
                        let mut target = rng.gen_range(0..100);
                        if target == src {
                            target = (target + 1) % 100;
                        }

                        (src, target)
                    })
                    .collect::<HashMap<_, _>>(),
            );

            tmp
        };

        let conf = termgraph::Config::new(termgraph::IDFormatter::new(), 20);

        b.iter(|| {
            let mut tmp = Vec::new();
            termgraph::fdisplay(black_box(&graph), &conf, &mut tmp);
        });
    });
}

pub fn linear_render(c: &mut Criterion) {
    c.bench_function("[linear] render 100 Nodes - 99 Edges", |b| {
        let graph = {
            let mut tmp = DirectedGraph::new();

            tmp.add_nodes((0..100).map(|idx| (idx, format!("test-{idx}"))));
            tmp.add_edges((0..99).map(|idx| (idx, idx + 1)));

            tmp
        };

        let conf = termgraph::Config::new(termgraph::IDFormatter::new(), 20);

        b.iter(|| {
            let mut tmp = Vec::new();
            termgraph::fdisplay(black_box(&graph), &conf, &mut tmp);
        });
    });
}

criterion_group!(benches, random_render, linear_render);
criterion_main!(benches);
