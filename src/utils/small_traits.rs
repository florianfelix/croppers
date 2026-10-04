use tracing::trace;

pub trait DimHelper {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn is_non_zero(&self) -> bool;
}

impl DimHelper for [u32; 2] {
    fn width(&self) -> u32 {
        self[0]
    }
    fn height(&self) -> u32 {
        self[1]
    }
    fn is_non_zero(&self) -> bool {
        let is_non_zero = self[0] > 0 && self[1] > 0;
        if is_non_zero {
            trace!("dims non zero: {:?}", self);
        } else {
            trace!("dims ZERO: {:?}", self);
        }
        is_non_zero
    }
}
