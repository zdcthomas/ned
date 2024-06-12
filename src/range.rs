/// Represents a point in text, column and row, 0 indexed
struct Point {
    x: usize,
    y: usize,
}

/// An actualized chunk of a buffer
///
/// TODO: should probably also contain the text that has been matched so far
pub struct Range {
    l: Point,
    r: Point,
}

impl Range {
    // TODO: <10-06-24, zdcthomas> Make Result when you figure out how errors work
    fn new(l: (usize, usize), r: (usize, usize)) -> Option<Self> {
        match (l, r) {
            ((lx, ly), (rx, ry)) if lx <= rx && ly <= ry => Some(Self {
                l: Point { x: lx, y: ly },
                r: Point { x: rx, y: ry },
            }),
            _ => None,
        }
    }
}
