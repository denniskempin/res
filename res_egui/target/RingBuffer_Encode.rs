impl < T, const N : usize > :: bincode :: Encode for RingBuffer < T, N > where
T : :: bincode :: Encode
{
    fn encode < __E : :: bincode :: enc :: Encoder >
    (& self, encoder : & mut __E) -> core :: result :: Result < (), :: bincode
    :: error :: EncodeError >
    { :: bincode :: Encode :: encode(& self.stack, encoder) ? ; Ok(()) }
}