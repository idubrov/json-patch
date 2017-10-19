extern crate test;
extern crate rand;

mod util;
mod generator;

use self::test::Bencher;
use self::rand::SeedableRng;
use super::{patch, patch_unsafe};

#[test]
fn tests() {
    util::run_specs("specs/tests.json");
}

#[test]
fn spec_tests() {
    util::run_specs("specs/spec_tests.json");
}

#[test]
fn revert_tests() {
    util::run_specs("specs/revert_tests.json");
}

#[bench]
fn bench_add_removes(b: &mut Bencher) {
    let mut rng = rand::StdRng::from_seed(&[0]);
    let params = generator::Params { ..Default::default() };
    let doc = params.gen(&mut rng);
    let patches = generator::gen_add_remove_patches(&doc, &mut rng, 10, 10);

    b.iter(|| {
        let mut doc = doc.clone();
        let mut result = Ok(());
        for ref p in &patches {
            // Patch mutable
            result = result.and_then(|_| patch(&mut doc, p));
        }
    });
}

#[bench]
fn bench_add_removes_unsafe(b: &mut Bencher) {
    let mut rng = rand::StdRng::from_seed(&[0]);
    let params = generator::Params { ..Default::default() };
    let doc = params.gen(&mut rng);
    let patches = generator::gen_add_remove_patches(&doc, &mut rng, 10, 10);

    b.iter(|| {
        let mut doc = doc.clone();
        let mut result = Ok(());
        for ref p in &patches {
            // Patch mutable
            result = unsafe { result.and_then(|_| patch_unsafe(&mut doc, p)) };
        }
    });
}