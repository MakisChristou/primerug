use std::io::{self, BufReader, BufWriter, Read, Write};
use std::os::unix::net::UnixStream;

pub struct GpuBatchResult {
    pub round_counts: Vec<u32>,
    pub survivor_indices: Vec<u32>,
}

pub struct GpuClient {
    reader: BufReader<UnixStream>,
    writer: BufWriter<UnixStream>,
    send_buf: Vec<u8>,
    recv_buf: Vec<u8>,
    batch_id: u32,
}

impl GpuClient {
    pub fn connect(socket_path: &str) -> io::Result<Self> {
        let stream = UnixStream::connect(socket_path)?;
        let read_stream = stream.try_clone()?;
        Ok(Self {
            reader: BufReader::new(read_stream),
            writer: BufWriter::new(stream),
            send_buf: Vec::with_capacity(8 * 1024 * 1024),
            recv_buf: Vec::with_capacity(256 * 1024),
            batch_id: 0,
        })
    }

    pub fn submit_batch(
        &mut self,
        limbs: &[u32],
        pattern: &[u64],
        limb_count: u16,
        num_candidates: u32,
    ) -> io::Result<GpuBatchResult> {
        self.batch_id = self.batch_id.wrapping_add(1);

        // Encode BatchRequest
        self.send_buf.clear();
        self.send_buf.extend_from_slice(&self.batch_id.to_le_bytes());
        self.send_buf.push(pattern.len() as u8);
        for &off in pattern {
            self.send_buf.extend_from_slice(&off.to_le_bytes());
        }
        self.send_buf
            .extend_from_slice(&limb_count.to_le_bytes());
        self.send_buf
            .extend_from_slice(&num_candidates.to_le_bytes());

        let limbs_bytes: &[u8] = bytemuck::cast_slice(limbs);
        self.send_buf.extend_from_slice(limbs_bytes);

        // Send length-prefixed message
        let len = self.send_buf.len() as u32;
        self.writer.write_all(&len.to_le_bytes())?;
        self.writer.write_all(&self.send_buf)?;
        self.writer.flush()?;

        // Read response
        let mut len_bytes = [0u8; 4];
        self.reader.read_exact(&mut len_bytes)?;
        let resp_len = u32::from_le_bytes(len_bytes) as usize;
        self.recv_buf.clear();
        self.recv_buf.resize(resp_len, 0);
        self.reader.read_exact(&mut self.recv_buf)?;

        // Decode BatchResult: batch_id(4) + num_rounds(4) + round_counts(4*N) + num_found(4) + indices(4*M)
        if self.recv_buf.len() < 8 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "response too short",
            ));
        }

        let mut pos = 4; // skip batch_id
        let num_rounds = u32::from_le_bytes(self.recv_buf[pos..pos + 4].try_into().unwrap()) as usize;
        pos += 4;

        let mut round_counts = Vec::with_capacity(num_rounds);
        for _ in 0..num_rounds {
            if pos + 4 > self.recv_buf.len() {
                break;
            }
            round_counts.push(u32::from_le_bytes(
                self.recv_buf[pos..pos + 4].try_into().unwrap(),
            ));
            pos += 4;
        }

        let num_found = if pos + 4 <= self.recv_buf.len() {
            u32::from_le_bytes(self.recv_buf[pos..pos + 4].try_into().unwrap()) as usize
        } else {
            0
        };
        pos += 4;

        let mut survivor_indices = Vec::with_capacity(num_found);
        for _ in 0..num_found {
            if pos + 4 > self.recv_buf.len() {
                break;
            }
            survivor_indices.push(u32::from_le_bytes(
                self.recv_buf[pos..pos + 4].try_into().unwrap(),
            ));
            pos += 4;
        }

        Ok(GpuBatchResult {
            round_counts,
            survivor_indices,
        })
    }
}
