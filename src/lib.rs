#[macro_export]
macro_rules! info {
    ($stream:expr, $data:literal) => {{
        let mut s = $stream;

        writeln!(&mut s, $data).unwrap();
    }};
}

#[cfg(test)]
mod tests {
    use std::io::{BufRead, BufReader, Error, ErrorKind, Read, Result, Write};
    use std::sync::mpsc::{self, Receiver, Sender};

    struct Duplex {
        tx: Sender<String>,
        rx: Receiver<String>,
    }

    impl Duplex {
        fn new() -> Self {
            let (tx, rx) = mpsc::channel();

            Self { tx, rx }
        }
    }

    impl Read for Duplex {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
            let data = self.rx.recv().unwrap();
            let bytes = data.as_bytes();

            for (i, val) in bytes.iter().enumerate() {
                if i >= buf.len() {
                    return Ok(buf.len());
                }
                buf[i] = val.clone();
            }

            // buf.clone_from_slice(bytes);

            Ok(bytes.len())
        }
    }

    impl Write for Duplex {
        fn write(&mut self, buf: &[u8]) -> Result<usize> {
            let data = String::from_utf8_lossy(buf).to_string();
            self.tx.send(data).unwrap();

            Ok(buf.len())
        }

        fn flush(&mut self) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn info() {
        let mut duplex = Duplex::new();
        info!(&mut duplex, "something");

        let mut reader = std::io::BufReader::new(duplex);

        let mut data = String::new();
        let result = reader.read_line(&mut data);

        assert_eq!(data, String::from("something\n"));
        assert_eq!(result.unwrap(), data.as_bytes().len());
    }
}
