// TODO: use use try_from everywhere

mod random;
mod urandom;

use std::io::{self, Read};

fn random() -> io::Result<u64> {
    let mut r = 0;

    for _ in 0..1024 {
        if unsafe { core::arch::x86_64::_rdrand64_step(&mut r) } == 1 {
            return Ok(r);
        }
    }

    Err(io::Error::new(
        io::ErrorKind::Other,
        "Failed to get random bits",
    ))
}

pub(crate) struct RandomReader;

impl Read for RandomReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let buf_len = buf.len();
        let mut buf_iter = buf.iter_mut();
        let mut read_count = 0;

        while read_count < buf_len {
            match random() {
                Ok(random) => {
                    for byte in random.to_ne_bytes() {
                        match buf_iter.next() {
                            None => break,
                            Some(next) => {
                                *next = byte;
                                read_count += 1;
                            }
                        }
                    }
                }
                Err(_e) => break,
            }
        }

        Ok(read_count)
    }
}

#[test]
fn random_reader_test() {
    const LENGTH: usize = 10_000;
    let mut random = RandomReader;
    let mut buffer = [0; LENGTH];
    assert_eq!(random.read(&mut buffer).unwrap(), LENGTH);
    assert_ne!(buffer, [0; LENGTH]);
}
