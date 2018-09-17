use std::io::Bytes;
use std::io::Read;

use failure::Error;
use twoway;

pub struct NalReader<R: Read> {
    inner: Bytes<R>,
}

impl<R: Read> NalReader<R> {
    pub fn new(inner: R) -> Self {
        NalReader {
            inner: inner.bytes(),
        }
    }

    pub fn read_nal(&mut self) -> Result<Option<Vec<u8>>, Error> {
        let mut first = match self.inner.next() {
            Some(byte) => byte?,
            None => return Ok(None),
        };

        let mut second = match self.inner.next() {
            Some(byte) => byte?,
            None => return Ok(Some(vec![first])),
        };

        let mut nal = Vec::with_capacity(4096);

        nal.push(first);
        nal.push(second);

        while let Some(byte) = self.inner.next() {
            let byte = byte?;
            if 0x00 == first && 0x00 == second {
                if 0x01 == byte {
                    // TODO: understand which specs allow an extra zero
                    // we don't want the two extra zeros in the array
                    nal.pop();
                    nal.pop();

                    break;
                }

                if 0x03 == byte {
                    second = 0x03;
                    continue;
                }

                // TODO: deny other values, for validation?
            }

            first = second;
            second = byte;

            nal.push(byte);
        }

        Ok(Some(nal))
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    use super::NalReader;

    fn nal_read(input: &[u8]) -> Vec<Vec<u8>> {
        let mut reader = NalReader::new(io::Cursor::new(input));
        let mut ret = Vec::new();
        while let Some(nal) = reader.read_nal().expect("reading nal") {
            ret.push(nal);
        }

        ret
    }

    #[test]
    fn no_terminator() {
        assert_eq!(vec![b"hello".to_vec()], nal_read(b"hello"));
        assert_eq!(
            vec![b"hello".to_vec(), b"bye".to_vec()],
            nal_read(b"hello\x00\x00\x01bye")
        );
        assert_eq!(vec![b"hello".to_vec()], nal_read(b"hello\x00\x00\x01"));
        assert_eq!(vec![vec![0u8; 0]], nal_read(&[0, 0, 1]));
        assert_eq!(vec![[0, 0].to_vec()], nal_read(&[0, 0, 3]));
        assert_eq!(vec![[0, 0, 7].to_vec()], nal_read(&[0, 0, 3, 7]));
    }
}
