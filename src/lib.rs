#![feature(read_initializer)]

// Naive run length encoding.

pub fn encode(input: &[u8]) -> Vec<u8> {
    let mut output = Vec::new();

    if input.len() < 1 {
        return output;
    }

    let mut count = 1;
    let mut byte = input[0];

    for &cur_byte in &input[1..] {
        if byte == cur_byte {
            // Same byte, increment count.
            count += 1;

            // If we hit the maximum count.
            if count == 255 {
                // Write repetition.
                output.push(count as u8);
                output.push(byte);

                // Reset count but the byte stays the same.
                count = 1;
            }
        } else {
            // Different byte, output repetition.
            output.push(count as u8);
            output.push(byte);

            // Reset count and change byte.
            count = 1;
            byte = cur_byte;
        }
    }

    // Flush.
    output.push(count as u8);
    output.push(byte);

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

#[derive(Debug)]
pub struct Encoder<R: io::Read> {
    input: R,
    buffer: Box<[u8]>,
    start: usize,
    end: usize,
    state: Option<(usize, u8)>,
    output_buffer: [u8; 2],
    output_buffer_count: u8,
}



impl<R: io::Read> Encoder<R> {
    pub fn with_capacity(capacity: usize, input: R) -> Self {
        let mut buffer = Vec::with_capacity(capacity);
        unsafe {
            buffer.set_len(capacity);
            input.initializer().initialize(&mut buffer);
        }
        Encoder {
            input,
            buffer: buffer.into_boxed_slice(),
            start: 0,
            end: 0,
            state: None,
            output_buffer: unsafe { std::mem::uninitialized() },
            output_buffer_count: 0,
        }
    }
}

impl<R: io::Read> io::Read for Encoder<R> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        let mut oi = 0;

        // Emit any buffered output data.
        while oi < output.len() && oi < self.output_buffer_count as usize {
            output[oi] = self.output_buffer[oi];
            oi += 1;
        }

        // Subtract number of bytes written from the amount of buffered output data.
        self.output_buffer_count -= oi as u8;

        // If the output buffer is not empty there is no need to continue.
        if self.output_buffer_count > 0 {
            return Ok(oi);
        }

        // TODO: We can decide to keep reading and processing until the output
        // buffer is full.

        // Check if we ran out of input data.
        if self.start == self.end {
            // Update end and then start only if read succeeds.
            self.end = self.input.read(&mut self.buffer)?;
            self.start = 0;
        }
        println!("{:?} {:?}", self.start, self.end);


        if self.start < self.end {

            // Initialize the state.
            let (mut count, mut byte) = if let Some((count, byte)) = self.state {
                // We have left overs from last time.
                self.state = None;
                (count, byte)
            } else {
                // We can use the first byte of the input to initialize the
                // state. We know that there is at least one byte since
                // self.start < self.end
                let ret = (1, self.buffer[self.start]);
                self.start += 1;
                ret
            };

            while self.start < self.end {
                let cur_byte = self.buffer[self.start];

                if cur_byte == byte {
                    count += 1;

                    if count == 255 {
                        if oi < output.len() {
                            output[oi] = count as u8;
                            oi += 1;
                        } else {
                            self.output_buffer[0] = count as u8;
                            self.output_buffer[1] = byte;
                            self.output_buffer_count = 2;
                            break;
                        }
                        if oi < output.len() {
                            output[oi] = byte;
                            oi += 1;
                        } else {
                            self.output_buffer[0] = byte;
                            self.output_buffer_count = 1;
                            break;
                        }

                        count = 1;
                    }
                } else {
                    if oi < output.len() {
                        output[oi] = count as u8;
                        oi += 1;
                    } else {
                        self.output_buffer[0] = count as u8;
                        self.output_buffer[1] = byte;
                        self.output_buffer_count = 2;
                        break;
                    }
                    if oi < output.len() {
                        output[oi] = byte;
                        oi += 1;
                    } else {
                        self.output_buffer[0] = byte;
                        self.output_buffer_count = 1;
                        break;
                    }

                    count = 1;
                    byte = cur_byte;
                }

                self.start += 1;
            }

            // Save the state.
            self.state = Some((count, byte));
        } else {
            // Flush the remaining state if it's there.
            if let Some((count, byte)) = self.state {
                if oi < output.len() {
                    output[oi] = count as u8;
                    oi += 1;
                } else {
                    self.output_buffer[0] = count as u8;
                    self.output_buffer[1] = byte;
                    self.output_buffer_count = 2;
                    return Ok(oi);
                }
                if oi < output.len() {
                    output[oi] = byte;
                    oi += 1;
                } else {
                    self.output_buffer[0] = byte;
                    self.output_buffer_count = 1;
                    return Ok(oi);
                }
            }
        }

        Ok(oi)
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

        let mut encoder = super::Encoder::with_capacity(10, &input[..]);

        let mut output = Vec::new();

        loop {
            let mut buf = vec![0xCC; 10];

            let bytes_read = encoder.read(&mut buf).unwrap();

            println!("{:?}", &buf[..bytes_read]);

            if bytes_read == 0 { break; }

            output.write(&buf[..bytes_read]).unwrap();
        }

        assert_eq!(&output, &vec![5, 0, 3, 3]);
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
