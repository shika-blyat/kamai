use std::ops::Range;
#[allow(dead_code)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

pub fn merge_ranges<T: Clone>(left_range: &Range<T>, right_range: &Range<T>) -> Range<T> {
    left_range.start.clone()..right_range.end.clone()
}
