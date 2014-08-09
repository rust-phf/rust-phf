use std::hash::{Hash, Hasher, Writer};
use std::hash::sip::{SipHasher, SipState};

static LOG_MAX_SIZE: uint = 21;

pub static MAX_SIZE: uint = 1 << LOG_MAX_SIZE;

pub fn displace(f1: u32, f2: u32, d1: u32, d2: u32) -> u32 {
    d2 + f1 * d1 + f2
}

fn split(hash: u64) -> (u32, u32, u32) {
    let mask = (MAX_SIZE - 1) as u64;

    ((hash & mask) as u32, 
     ((hash >> LOG_MAX_SIZE) & mask) as u32,
     ((hash >> (2 * LOG_MAX_SIZE)) & mask) as u32)
}

pub trait PhfHash {
    fn phf_hash(&self, seed: u64) -> (u32, u32, u32);
}

impl<'a> PhfHash for &'a str {
    fn phf_hash(&self, seed: u64) -> (u32, u32, u32) {
        split(SipHasher::new_with_keys(0, seed).hash(self))
    }
}

impl<'a> PhfHash for &'a [u8] {
    fn phf_hash(&self, seed: u64) -> (u32, u32, u32) {
        let mut state = SipState::new_with_keys(0, seed);
        state.write(*self);
        split(state.result())
    }
}

macro_rules! sip_impl(
    ($t:ty) => (
        impl PhfHash for $t {
            fn phf_hash(&self, seed: u64) -> (u32, u32, u32) {
                split(SipHasher::new_with_keys(0, seed).hash(self))
            }
        }
    )
)

sip_impl!(u8)
sip_impl!(i8)
sip_impl!(u16)
sip_impl!(i16)
sip_impl!(u32)
sip_impl!(i32)
sip_impl!(u64)
sip_impl!(i64)
sip_impl!(char)
sip_impl!(bool)
