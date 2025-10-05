pub enum MyError<E> {
    IO(E),
    Deserialize,
    Serialize,
    BufferFull,
    Unkown
}