pub struct Left;
pub struct Right;

pub trait Side {
    type Other: Side + 'static;
    fn get<T>(pair: (T, T)) -> T;
    fn get_mut<T>(pair: &mut (T, T)) -> &mut T;
    fn temple() -> u32;
}

impl Side for Left {
    type Other = Right;
    fn get<T>(pair: (T, T)) -> T {
        pair.0
    }

    fn get_mut<T>(pair: &mut (T, T)) -> &mut T {
        &mut pair.0
    }

    fn temple() -> u32 {
        1 << 16
    }
}

impl Side for Right {
    type Other = Left;
    fn get<T>(pair: (T, T)) -> T {
        pair.1
    }

    fn get_mut<T>(pair: &mut (T, T)) -> &mut T {
        &mut pair.1
    }

    fn temple() -> u32 {
        1 << 12
    }
}
