use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use mcseedcracker::features::end_pillars::{EndPillars, PartialEndPillars, PillarHeightHint};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

fn reverse_pillar_seed_single_threaded(input: &PartialEndPillars, real_seed: i64) -> i64 {
    let mut seed = None;
    let mut rpillars = EndPillars::new();
    for pseed in 0..65536 {
        rpillars.from_seed(pseed);
        if !input.matches(&rpillars).is_impossible_match() {
            if let Some(s) = seed {
                panic!("Found two pillar seeds: {} and {}", s, pseed);
            }
            seed = Some(pseed);
        }
    }

    let found = seed.expect("No pillar seed found");
    assert_eq!(
        found, real_seed,
        "Wrong seed, found {found}, expected {real_seed}"
    );
    found
}

fn reverse_pillar_seed_rayon(input: &PartialEndPillars, real_seed: i64) -> i64 {
    let results = (0..65536)
        .into_par_iter()
        .filter_map(|seed| {
            let mut rpillars = EndPillars::new();
            rpillars.from_seed(seed);
            if !input.matches(&rpillars).is_impossible_match() {
                Some(seed)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    match &results[..] {
        [] => panic!("No pillar seed found"),
        [seed] => {
            assert_eq!(
                *seed, real_seed,
                "Wrong seed, found {}, expected {real_seed}",
                *seed
            );
            *seed
        }
        _ => panic!("Found two pillar seeds: {} and {}", results[0], results[1]),
    }
}

fn reverse_pillar_seed(c: &mut Criterion) {
    let mut group = c.benchmark_group("reverse_pillar_seed");

    for seed in [
        57809, 28025, 12710, 28642, 16896, 53275, 4002, 33676, 58959, 21677,
    ] {
        let mut real_pillars = EndPillars::new();
        real_pillars.from_seed(seed);

        let mut partial_input = PartialEndPillars::new();
        for (ppillar, pillar) in partial_input.iter_mut().zip(real_pillars.iter()) {
            if pillar.caged {
                ppillar.caged = Some(true);
                ppillar.height = PillarHeightHint::Exact(pillar.height);
            } else if pillar.height == 103
                || pillar.height == 100
                || pillar.height == 97
                || pillar.height == 94
                || pillar.height == 76
            {
                ppillar.height = PillarHeightHint::Exact(pillar.height);
            }
        }

        group.bench_with_input(
            BenchmarkId::new("single_threaded", format!("seed={seed}")),
            &(partial_input, seed),
            |b, input| b.iter(|| reverse_pillar_seed_single_threaded(&input.0, input.1)),
        );
        group.bench_with_input(
            BenchmarkId::new("threaded_rayon", format!("seed={seed}")),
            &(partial_input, seed),
            |b, input| b.iter(|| reverse_pillar_seed_rayon(&input.0, input.1)),
        );
    }

    group.finish();
}

criterion_group!(benches, reverse_pillar_seed);
criterion_main!(benches);
