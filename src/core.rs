use std::{str::FromStr, convert::Infallible};

use ropey::Rope;

pub struct Core {
    pub rope: Rope,
}

impl FromStr for Core {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            rope: Rope::from_str(s)
        })
    }
}


#[cfg(test)]
mod test {
    use super::Core;
    use ropey::Rope;
    use std::str::FromStr;

    #[test]
    fn test_from_str() {
        let _rope = Core::from_str("Hello world!");
    }

    #[test]
    fn rope_test() {
        let mut rope = Rope::from_str("Hello world!");

        rope.remove(6..11);
        rope.insert(6, "Boy");

        assert_eq!(rope, "Hello Boy!");

        rope.insert(6, "\nBlah die blah");

        dbg!(rope.len_chars());
        dbg!(rope.len_lines());
        dbg!(rope.len_utf16_cu());

        // rope.try_remove(0..200).expect("out of bounds");
    }
}
