use std::{
    cmp::Ordering,
    fmt::{Debug, Display},
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Eval(pub i8);

impl Display for Eval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0.cmp(&0) {
            Ordering::Greater => write!(f, "Win: {}", (-self.0) - i8::MIN),
            Ordering::Less => write!(f, "Loss: {}", self.0 - i8::MIN),
            Ordering::Equal => write!(f, "Tie"),
        }
    }
}

impl Debug for Eval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self, f)
    }
}

impl Eval {
    #[inline]
    pub fn new_win(steps: i8) -> Self {
        assert!((1..=i8::MAX).contains(&steps));
        Eval(-(i8::MIN + steps))
    }

    #[inline]
    pub fn new_loss(steps: i8) -> Self {
        assert!((0..=i8::MAX).contains(&steps));
        Eval(i8::MIN + steps)
    }

    #[inline]
    pub fn new_tie() -> Self {
        Eval(0)
    }

    #[inline]
    pub fn backward(self) -> Self {
        debug_assert!(self.0 != -1);
        match self.0.cmp(&0) {
            Ordering::Less => Eval(-(self.0 + 1)),
            Ordering::Equal => Eval(0),
            Ordering::Greater => Eval(-self.0),
        }
    }

    #[inline]
    pub fn forward(self) -> Self {
        debug_assert!(self.0 != i8::MIN);
        match self.0.cmp(&0) {
            Ordering::Less => Eval(-self.0),
            Ordering::Equal => Eval(0),
            Ordering::Greater => Eval((-self.0) - 1),
        }
    }

    #[inline]
    pub fn plies(self) -> u8 {
        match self.0.cmp(&0) {
            Ordering::Less => (self.0 - i8::MIN) as u8 * 2,
            Ordering::Equal => u8::MAX,
            Ordering::Greater => ((-self.0) - i8::MIN) as u8 * 2 - 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::*;

    #[test]
    fn test_eval_size() {
        assert_eq!(size_of::<Eval>(), 1)
    }

    #[test]
    fn test_eval_cmp() {
        assert!(Eval::new_loss(10) > Eval::new_loss(8));
        assert!(Eval::new_tie() > Eval::new_loss(10));
        assert!(Eval::new_tie() > Eval::new_loss(0));
        assert!(Eval::new_win(10) > Eval::new_tie());
        assert!(Eval::new_win(1) > Eval::new_tie());
        assert!(Eval::new_win(10) > Eval::new_loss(10));
        assert!(Eval::new_win(8) > Eval::new_win(10));
    }

    #[test]
    fn test_eval_display() {
        assert_eq!("Loss: 5", format!("{}", Eval::new_loss(5)));
        assert_eq!("Loss: 1", format!("{}", Eval::new_loss(1)));
        assert_eq!("Loss: 0", format!("{}", Eval::new_loss(0)));
        assert_eq!("Win: 5", format!("{}", Eval::new_win(5)));
        assert_eq!("Win: 1", format!("{}", Eval::new_win(1)));
        assert_eq!("Tie", format!("{}", Eval::new_tie()));
    }

    #[test]
    fn test_eval_plies() {
        assert_eq!(10, Eval::new_loss(5).plies());
        assert_eq!(0, Eval::new_loss(0).plies());
        assert_eq!(9, Eval::new_win(5).plies());
        assert_eq!(1, Eval::new_win(1).plies());
        assert_eq!(255, Eval::new_tie().plies());
    }

    #[test]
    fn forward_backward() {
        for i in -127..=127 {
            assert_eq!(Eval(i).forward().backward(), Eval(i))
        }
    }
}
