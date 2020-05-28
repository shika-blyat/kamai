#![allow(unused)]

pub enum Either<L, R> {
    Left(L),
    Right(R),
}
