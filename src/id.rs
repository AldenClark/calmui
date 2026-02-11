#[track_caller]
pub fn stable_auto_id(prefix: &str) -> String {
    let location = std::panic::Location::caller();
    let seed = format!(
        "{prefix}:{}:{}:{}",
        location.file(),
        location.line(),
        location.column()
    );
    format!("{prefix}-{:016x}", fnv1a64(seed.as_bytes()))
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    const OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x00000100000001b3;

    let mut hash = OFFSET_BASIS;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[track_caller]
    fn call_once() -> String {
        stable_auto_id("button")
    }

    #[test]
    fn id_is_stable_for_same_callsite() {
        let ids = (0..3).map(|_| call_once()).collect::<Vec<_>>();
        assert!(ids.windows(2).all(|pair| pair[0] == pair[1]));
    }

    #[test]
    fn id_differs_for_different_callsites() {
        let first = call_once();
        let second = {
            // Different callsite by design.
            stable_auto_id("button")
        };
        assert_ne!(first, second);
    }
}
