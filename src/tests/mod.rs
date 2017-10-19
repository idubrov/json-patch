extern crate test;
extern crate rand;

mod util;
mod generator;

use self::test::Bencher;
use self::rand::SeedableRng;
use super::{patch, patch_mut, patch_unsafe};

#[test]
fn tests() {
    util::run_specs("specs/tests.json");
}

#[test]
fn spec_tests() {
    util::run_specs("specs/spec_tests.json");
}

#[bench]
fn bench_add_removes(b: &mut Bencher) {
    let mut rng = rand::StdRng::from_seed(&[0]);
    let params = generator::Params { ..Default::default() };
    let doc = params.gen(&mut rng);
    let patches = generator::gen_add_remove_patches(&doc, &mut rng, 10, 10);

    b.iter(|| {
        let mut doc = Ok(doc.clone());
        for ref p in &patches {
            // Patch immutable
            doc = doc.and_then(|d| patch(&d, p));
        }
    });
}


#[bench]
fn bench_add_removes_mut(b: &mut Bencher) {
    let mut rng = rand::StdRng::from_seed(&[0]);
    let params = generator::Params { ..Default::default() };
    let doc = params.gen(&mut rng);
    let patches = generator::gen_add_remove_patches(&doc, &mut rng, 10, 10);

    b.iter(|| {
        let mut doc = doc.clone();
        let mut result = Ok(());
        for ref p in &patches {
            // Patch mutable
            result = result.and_then(|_| patch_mut(&mut doc, p));
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
        let mut d = doc.clone();
        let mut result = Ok(());
        for ref p in &patches {
            // Patch unsafe (mutable without rollback)
            result = result.and_then(|_| unsafe { patch_unsafe(&mut d, p) });
        }
    });
}
