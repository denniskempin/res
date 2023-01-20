impl < T, const N : usize > :: bincode :: Decode for RingBuffer < T, N > where
T : :: bincode :: Decode
{
    fn decode < __D : :: bincode :: de :: Decoder > (decoder : & mut __D) ->
    core :: result :: Result < Self, :: bincode :: error :: DecodeError >
    { Ok(Self { stack : :: bincode :: Decode :: decode(decoder) ?, }) }
} impl < '__de, T, const N : usize > :: bincode :: BorrowDecode < '__de > for
RingBuffer < T, N > where T : :: bincode :: de :: BorrowDecode < '__de >
{
    fn borrow_decode < __D : :: bincode :: de :: BorrowDecoder < '__de > >
    (decoder : & mut __D) -> core :: result :: Result < Self, :: bincode ::
    error :: DecodeError >
    {
        Ok(Self
        { stack : :: bincode :: BorrowDecode :: borrow_decode(decoder) ?, })
    }
}