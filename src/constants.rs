use crate::FixedDec;

const PI_128DIGITS: u128 = 3_141_592_653_589_793_238_462_643_383_279_502_884_19;
const PI_64DIGITS: u64 = 3_141_592_653_589_793_238;
const PI_32DIGITS: u32 = 3_141_592_653;

pub const PI128: FixedDec<u128, 38> = FixedDec::new(PI_128DIGITS);
pub const PI64: FixedDec<u64, 18> = FixedDec::new(PI_64DIGITS);
pub const PI32: FixedDec<u32, 9> = FixedDec::new(PI_32DIGITS);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pies() {
        let pi128 = alloc::format!("{}", PI128);
        let pi64 = alloc::format!("{}", PI64);
        let pi32 = alloc::format!("{}", PI32);
        let ref_pi = "3.14159265358979323846264338327950288419";

        assert!(pi128.starts_with(ref_pi));
        assert!(pi128.starts_with(&pi64));
        assert!(pi128.starts_with(&pi32));

        assert_eq!(PI64.set_precision::<0>(), Some(FixedDec::new(3)));
        assert_eq!(PI64.set_precision::<1>(), Some(FixedDec::new(31)));
        assert_eq!(PI64.set_precision::<2>(), Some(FixedDec::new(314)));
        assert_eq!(PI64.set_precision::<3>(), Some(FixedDec::new(3141)));
        assert_eq!(PI64.set_precision::<4>(), Some(FixedDec::new(31415)));
    }
}
