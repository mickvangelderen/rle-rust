// Naive run length encoding.

pub fn encode(input: &[u8]) -> Vec<u8> {
    let mut output = Vec::new();

    let mut i = 0;
    while i < input.len() {
        let byte_i = input[i];

        let mut j = i + 1;
        while j < input.len() && (j - i) < 256 {
            let byte_j = input[j];
            if byte_j != byte_i { break }
            j += 1;
        }

        output.push((j - i) as u8);
        output.push(byte_i);
        i = j;
    }

    output
}

// TODO: output overflow handling
pub fn encode_into(input: &[u8], output: &mut [u8]) -> usize {
    let mut o = 0;
    let mut i = 0;

    while i < input.len() {
        let byte_i = input[i];

        let mut j = i + 1;
        while j < input.len() && (j - i) < 256 {
            let byte_j = input[j];
            if byte_j != byte_i { break }
            j += 1;
        }

        output[o] = (j - i) as u8;
        output[o + 1] = byte_i;
        o += 2;

        i = j;
    }

    o
}

use std::io;

pub struct Encoder<R: io::Read> {
    input: R
}

impl<R: io::Read> io::Read for Encoder<R> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        let mut buffer = Vec::with_capacity(output.len()/2);
        unsafe { buffer.set_len(output.len()/2) };

        let bytes_read = self.input.read(&mut buffer)?;
        unsafe { buffer.set_len(bytes_read); }

        Ok(encode_into(&buffer, output))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(
            super::encode(&[1, 2, 3, 4, 4, 4, 5]),
            &[1, 1, 1, 2, 1, 3, 3, 4, 1, 5]
        );
    }

    #[test]
    fn enc() {
        use std::io::Read;

        let input: Vec<u8> = vec![0, 0, 0, 0, 0, 3, 3, 3];

        let mut enc = super::Encoder { input: &input[..] };

        let mut buf = vec![0xCC; 10];

        assert_eq!(
            enc.read(&mut buf).unwrap(),
            2
        );

        assert_eq!(
            &buf,
            &[5, 0, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC]
        );

        let mut buf = vec![0xCC; 2];
        assert_eq!(
            enc.read(&mut buf).unwrap(),
            2
        );

        assert_eq!(
            &buf,
            &[1, 3]
        );
    }

    use std::io;
    use std::io::Read;
    use std::io::Write;

    #[test]
    fn test_encode_pipe() {
        let input: Vec<u8> = vec![1, 2, 2, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 5, 5, 5, 5, 5];
        let expected: Vec<u8> = vec![1, 1, 2, 2, 10, 10, 5, 5];
        let mut output: Vec<u8> = Vec::with_capacity(expected.len());
        encode_pipe(&mut &input[..], &mut output).unwrap();
        assert_eq!(output, expected);
    }

    fn encode_pipe<R: Read, W: Write>(input: &mut R, output: &mut W) -> io::Result<()> {
        let mut state: Option<(usize, u8)> = None;

        loop {
            let mut buffer = vec![0; 10];
            let len = input.read(&mut buffer)?;
            if len == 0 { break; }

            let mut i = 0;

            if let Some((count, byte)) = state {
                // Clear the state.
                state = None;

                let mut j = 0;
                while j < len && (count + j) < 256 {
                    let byte_j = buffer[j];

                    if byte_j != byte { break; }

                    j += 1;
                }

                if j == len {
                    // Save state.
                    state = Some((count + j, byte));
                } else {
                    // Write count and byte.
                    output.write(&[(count + j) as u8, byte])?;
                }
                i = j;
            }

            while i < len {
                let byte_i = buffer[i];

                let mut j = i + 1;
                while j < len && (j - i) < 256 {
                    let byte_j = buffer[j];

                    if byte_j != byte_i { break; }

                    j += 1;
                }

                if j == len {
                    // Save state.
                    state = Some((j - i, byte_i));
                } else {
                    // Write count and byte.
                    output.write(&[(j - i) as u8, byte_i])?;
                }

                i = j;
            }
        }

        // Flush the state.
        if let Some((count, byte)) = state {
            // Should always be true, except when the input length is 0.
            output.write(&[count as u8, byte])?;
        }

        Ok(())
    }
}
