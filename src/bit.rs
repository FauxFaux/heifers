use std::io;
use std::io::Read;

use cast::u16;
use cast::u32;
use cast::u64;
use cast::u8;
use cast::usize;
pub use generic_array::typenum;
use generic_array::ArrayLength;
use generic_array::GenericArray;

pub struct Bits<N: ArrayLength<u8>> {
    data: GenericArray<u8, N>,
    pos: usize,
}

impl<N: ArrayLength<u8>> Bits<N> {
    pub fn read_exact<R: Read>(mut from: R) -> Result<Bits<N>, io::Error> {
        let mut data = GenericArray::<u8, N>::default();
        from.read_exact(data.as_mut_slice())?;
        Ok(Bits { data, pos: 0 })
    }

    pub fn read_bool(&mut self) -> bool {
        let byte = self.pos / 8;
        let mask = 1 << (7 - (self.pos % 8));
        self.pos += 1;
        self.data[byte] & mask == mask
    }

    pub fn read_bits(&mut self, bits: u8) -> u64 {
        let mut ret = 0;

        for i in (0..bits).rev() {
            if self.read_bool() {
                ret |= 1 << i;
            }
        }

        ret
    }

    pub fn read_u8(&mut self, bits: u8) -> u8 {
        assert_le!(bits, 8);
        u8(self.read_bits(bits)).unwrap()
    }

    pub fn read_u16(&mut self, bits: u8) -> u16 {
        assert_le!(bits, 16);
        u16(self.read_bits(bits)).unwrap()
    }

    pub fn read_u32(&mut self, bits: u8) -> u32 {
        assert_le!(bits, 32);
        u32(self.read_bits(bits)).unwrap()
    }

    pub fn read_u64(&mut self, bits: u8) -> u64 {
        assert_le!(bits, 64);
        u64(self.read_bits(bits))
    }

    pub fn skip(&mut self, bits: u8) -> &mut Self {
        self.pos += usize(bits);
        self
    }

    pub fn done(&self) -> bool {
        self.pos == self.data.len() * 8
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use byteorder::WriteBytesExt;
    use generic_array::typenum;

    use bit::Bits;
    use byteorder::BE;

    #[test]
    fn one_byte() {
        let c = Cursor::new([0b1100_0100u8]);
        let mut bits = Bits::<typenum::U1>::read_exact(c).expect("reading from cursor");
        assert!(bits.read_bool());
        assert!(bits.read_bool());
        assert!(!bits.read_bool());
        assert!(!bits.read_bool());
        assert!(!bits.read_bool());
        assert!(bits.read_bool());
        assert!(!bits.read_bool());
        assert!(!bits.read_bool());
    }

    #[test]
    fn sub_byte() {
        let c = Cursor::new([0b1100_0100u8]);
        let mut bits = Bits::<typenum::U1>::read_exact(c).expect("reading from cursor");
        assert_eq!(0b1100, bits.read_u8(4));
        assert_eq!(0b01, bits.read_u8(2));
        assert_eq!(0b0, bits.read_u8(2));
    }

    #[test]
    fn multiple_bytes() {
        let c = Cursor::new(b"abc");
        let mut bits = Bits::<typenum::U3>::read_exact(c).expect("reading from cursor");

        assert_eq!(b'a', bits.read_u8(8));
        assert_eq!(b'b', bits.read_u8(8));
        assert_eq!(b'c', bits.read_u8(8));
    }

    #[test]
    fn multiple_word() {
        let mut buf = Vec::new();
        buf.write_u32::<BE>(987654321).unwrap();
        let mut bits =
            Bits::<typenum::U4>::read_exact(Cursor::new(buf)).expect("reading from cursor");
        assert_eq!(987654321, bits.read_u32(32));
    }
}
