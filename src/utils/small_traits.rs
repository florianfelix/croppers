pub trait DimHelper {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
}

impl DimHelper for [u32; 2] {
    fn width(&self) -> u32 {
        self[0]
    }
    fn height(&self) -> u32 {
        self[1]
    }
}
