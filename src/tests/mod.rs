extern crate test;
extern crate rand;

mod util;
mod generator;

use self::test::Bencher;
use self::rand::SeedableRng;
use super::patch_unsafe;


#[test]
fn tests() {
    util::run_specs("specs/tests.json");
}

#[test]
fn spec_tests() {
    util::run_specs("specs/spec_tests.json");
}

#[bench]
fn bench_removes(b: &mut Bencher) {
    let mut rng = rand::StdRng::new().unwrap();
    rng.reseed(&[0]);
    let params = generator::Params{ ..Default::default() };
    let doc = params.gen(&mut rng);
    let paths: Vec<String> = (0..10).map(|_| generator::gen_path(&doc, &mut rng)).collect();

    b.iter(|| {
        let mut d = doc.clone();
        for ref path in &paths {
            let arr = [super::PatchOperation::Remove(super::RemoveOperation { path: (*path).clone() })];
            unsafe { patch_unsafe(&mut d, &arr).unwrap(); }
        }
    });
}
