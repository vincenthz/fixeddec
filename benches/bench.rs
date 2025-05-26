//use divan::{Bencher, counter::BytesCount};
use fixeddec::FixedDec;

fn main() {
    divan::main()
}

#[divan::bench(args = [1, 10, 1000])]
fn from_integral(arg: u32) -> Option<FixedDec<u32, 3>> {
    FixedDec::<u32, 3>::from_integral(arg)
}

#[divan::bench(args = [0.1, 0.2, 0.3])]
fn add_float(arg: f32) -> f32 {
    0.1 + arg
}

#[divan::bench(args = [1, 2, 3])]
fn add_fixeddec(arg: u32) -> FixedDec<u32, 1> {
    FixedDec::new(1) + FixedDec::new(arg)
}

#[divan::bench(args = [1, 2, 3])]
fn fixeddec_integral(arg: u32) -> u32 {
    let f = FixedDec::<u32, 3>::new(2000);
    (f + FixedDec::new(arg)).integral()
}

#[divan::bench(args = [0.001, 0.002, 0.003])]
fn float_integral(arg: f32) -> u32 {
    let f = 2.0f32;
    (f + arg) as u32
}
