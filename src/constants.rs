use crate::FixedDec;

const PI_128DIGITS: u128 = 3_141_592_653_589_793_238_462_643_383_279_502_884_19;
const PI_64DIGITS: u64 = 3_141_592_653_589_793_238;
const PI_32DIGITS: u32 = 3_141_592_653;

pub const PI128: FixedDec<u128, 38> = FixedDec::new(PI_128DIGITS);
pub const PI64: FixedDec<u64, 18> = FixedDec::new(PI_64DIGITS);
pub const PI32: FixedDec<u32, 9> = FixedDec::new(PI_32DIGITS);

const E_128DIGITS: u128 = 2_718_281_828_459_045_235_360_287_471_352_662_497_75;
const E_64DIGITS: u64 = 2_718_281_828_459_045_235;
const E_32DIGITS: u32 = 2_718_281_828;

pub const E128: FixedDec<u128, 38> = FixedDec::new(E_128DIGITS);
pub const E64: FixedDec<u64, 18> = FixedDec::new(E_64DIGITS);
pub const E32: FixedDec<u32, 9> = FixedDec::new(E_32DIGITS);

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

    #[test]
    fn es() {
        let e128 = alloc::format!("{}", E128);
        let e64 = alloc::format!("{}", E64);
        let e32 = alloc::format!("{}", E32);
        let ref_e = "2.71828182845904523536028747135266249775";

        assert!(e128.starts_with(ref_e));
        assert!(e128.starts_with(&e64));
        assert!(e128.starts_with(&e32));

        assert_eq!(E64.set_precision::<0>(), Some(FixedDec::new(2)));
        assert_eq!(E64.set_precision::<1>(), Some(FixedDec::new(27)));
        assert_eq!(E64.set_precision::<2>(), Some(FixedDec::new(271)));
        assert_eq!(E64.set_precision::<3>(), Some(FixedDec::new(2718)));
        assert_eq!(E64.set_precision::<4>(), Some(FixedDec::new(27182)));
    }
}
