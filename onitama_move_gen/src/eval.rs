use std::{cmp::Ordering, fmt::Display, num::NonZeroU8};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Eval(NonZeroU8);

impl Display for Eval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tie = i8::MAX as u8;
        let val = self.0.get();
        match val.cmp(&tie) {
            Ordering::Greater => write!(f, "Win: {}", val.wrapping_neg()),
            Ordering::Less => write!(f, "Loss: {}", val),
            Ordering::Equal => write!(f, "Tie"),
        }
    }
}

impl Eval {
    pub fn new_win(steps: u8) -> Self {
        assert!(steps < i8::MAX as u8);
        Eval(NonZeroU8::new(steps.wrapping_neg()).unwrap())
    }

    pub fn new_loss(steps: u8) -> Self {
        assert!(steps < i8::MAX as u8);
        Eval(NonZeroU8::new(steps).unwrap())
    }

    pub fn new_tie() -> Self {
        Eval(NonZeroU8::new(i8::MAX as u8).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::*;

    #[test]
    fn test_eval_size() {
        assert_eq!(size_of::<Option<Eval>>(), 1)
    }

    #[test]
    fn test_eval_cmp() {
        assert!(Eval::new_loss(10) > Eval::new_loss(8));
        assert!(Eval::new_tie() > Eval::new_loss(10));
        assert!(Eval::new_win(10) > Eval::new_tie());
        assert!(Eval::new_win(10) > Eval::new_loss(10));
        assert!(Eval::new_win(8) > Eval::new_win(10));
    }

    #[test]
    fn test_eval_display() {
        assert_eq!("Loss: 5", format!("{}", Eval::new_loss(5)));
        assert_eq!("Win: 5", format!("{}", Eval::new_win(5)));
        assert_eq!("Tie", format!("{}", Eval::new_tie()));
    }
}
