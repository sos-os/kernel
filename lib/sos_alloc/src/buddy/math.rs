#[cfg(target_arch="x86")]
const B: [usize; 5]
    = [ 0x2, 0xC, 0xF0
      , 0xFF00
      , 0xFFFF0000];
#[cfg(target_arch="x86")]
const S: [usize; 5] = [ 1, 2, 4, 8, 16 ];

#[cfg(target_arch="x86_64")]
const B: [usize; 6]
    = [ 0x2, 0xC, 0xF0
      , 0xFF00, 0xFFFF0000
      , 0xFFFFFFFF00000000];

#[cfg(target_arch="x86_64")]
const S: [usize; 6] = [ 1, 2, 4, 8, 16, 32 ];

pub trait PowersOf2 {
    fn is_pow2(&self) -> bool;
    fn next_pow2(&self) -> Self;
    fn log2(&self) -> Self;
}

impl PowersOf2 for usize {
    fn is_pow2(&self) -> bool {
        *self != 0 && (self & (self - 1)) == 0
    }

    /// Returns the next power of 2
    fn next_pow2(&self) -> usize {
        let mut v = *self;
        if v == 0 {
            1
        } else {
            v -= 1;
            v = v | (v >> 1);
            v = v | (v >> 2);
            v = v | (v >> 4);
            v = v | (v >> 8);
            v = v | (v >> 16);
            v + 1
        }
    }

    #[cfg(not(any(target_arch="x86_64", target_arch="x86")))]
    fn log2(&self) -> usize {
        // This is the "obvious" log base 2 implementation. The lookup table
        // -based approach would be much faster, but we can't use it for
        // `usize` since we aren't sure what size a usize is without conditional
        // compilation.
        let mut res = 0;
        let mut num = *self >> 1;
        while num != 0 {
            res += 1;
            num >>= 1;
        }
        res
    }

    #[cfg(any(target_arch="x86_64", target_arch="x86"))]
    fn log2(&self) -> usize {
        let mut r: usize = 0;
        let mut v = *self;
        // S.iter()
        //  .zip(B.iter()) // this purely-functional implementation may be slower
        //  .fold((0, *self), // but it's cute and I had fun figuring it out
        //     |(r, v), (s, b)| if v & b != 0 { (r | s, v >> s) }
        //                      else { (r, v) })
        //   .0
        for i in (0..S.len()).rev() {
            if v & B[i] != 0 {
                v >>= S[i];
                r |=  S[i];
            }
        }
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_pow2() {
        assert_eq!(2usize.next_pow2(), 2);
        assert_eq!(0usize.next_pow2(), 1);
        assert_eq!(3usize.next_pow2(), 4);
        assert_eq!(5678usize.next_pow2(), 8192);
    }
}
