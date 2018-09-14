use std::io::BufRead;

use failure::Error;
use twoway;

struct NalReader<R: BufRead> {
    inner: R,
}

impl<R: BufRead> NalReader<R> {
    pub fn new(inner: R) -> Self {
        NalReader { inner }
    }

    pub fn read_nal(&mut self) -> Result<Option<Vec<u8>>, Error> {
        if self.inner.fill_buf()?.is_empty() {
            return Ok(None);
        }

        let mut nal = Vec::with_capacity(4096);
        loop {
            enum Action {
                Consume,
                Break,
                CheckProgress,
            }

            let (action, consume) = {
                let buf = self.inner.fill_buf()?;

                match buf.len() {
                    0 => break,
                    1 | 2 => (Action::CheckProgress, buf.len()),
                    _ => if let Some(end) = twoway::find_bytes(buf, &[0, 0, 1]) {
                        nal.extend(&buf[..end]);
                        (Action::Break, end + 3)
                    } else {
                        let safe = buf.len() - 2;
                        nal.extend(&buf[..safe]);
                        (Action::Consume, safe)
                    },
                }
            };

            match action {
                Action::Break => {
                    self.inner.consume(consume);
                    break;
                }
                Action::Consume => {
                    self.inner.consume(consume);
                }
                Action::CheckProgress => {
                    let remaining = {
                        let buf = self.inner.fill_buf()?;
                        if buf.len() == consume {
                            // no further progress, we're at the end of the file
                            // can't contain a marker as it's too short
                            nal.extend(buf);
                            buf.len()
                        } else {
                            0
                        }
                    };
                    self.inner.consume(remaining);
                }
            }
        }

        while let Some(pos) = twoway::rfind_bytes(&nal, &[0, 0, 3]) {
            nal.remove(pos + 2);
        }

        Ok(Some(nal))
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    use super::NalReader;

    #[test]
    fn no_terminator() {
        NalReader::new(io::Cursor::new(b"hello"))
            .read_nal()
            .unwrap();
    }
}
