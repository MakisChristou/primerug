use std::io::{self, BufReader, BufWriter, Read, Write};
use std::net::TcpStream;
use std::os::unix::net::UnixStream;

pub struct GpuBatchResult {
    pub round_counts: Vec<u32>,
    pub survivor_indices: Vec<u32>,
}

const TAG_COMPACT: u8 = 0x02;

enum Transport {
    Unix {
        reader: BufReader<UnixStream>,
        writer: BufWriter<UnixStream>,
    },
    Tcp {
        reader: BufReader<TcpStream>,
        writer: BufWriter<TcpStream>,
    },
}

pub struct GpuClient {
    transport: Transport,
    send_buf: Vec<u8>,
    recv_buf: Vec<u8>,
    batch_id: u32,
}

impl GpuClient {
    /// Connect via Unix socket.
    pub fn connect(socket_path: &str) -> io::Result<Self> {
        let stream = UnixStream::connect(socket_path)?;
        let read_stream = stream.try_clone()?;
        Ok(Self {
            transport: Transport::Unix {
                reader: BufReader::new(read_stream),
                writer: BufWriter::new(stream),
            },
            send_buf: Vec::with_capacity(256 * 1024),
            recv_buf: Vec::with_capacity(256 * 1024),
            batch_id: 0,
        })
    }

    /// Connect via TCP.
    pub fn connect_tcp(addr: &str) -> io::Result<Self> {
        let stream = TcpStream::connect(addr)?;
        stream.set_nodelay(true)?;
        let read_stream = stream.try_clone()?;
        Ok(Self {
            transport: Transport::Tcp {
                reader: BufReader::new(read_stream),
                writer: BufWriter::new(stream),
            },
            send_buf: Vec::with_capacity(256 * 1024),
            recv_buf: Vec::with_capacity(256 * 1024),
            batch_id: 0,
        })
    }

    /// Auto-detect: if address contains ':' use TCP, otherwise Unix socket.
    pub fn connect_auto(addr: &str) -> io::Result<Self> {
        if addr.contains(':') {
            Self::connect_tcp(addr)
        } else {
            Self::connect(addr)
        }
    }

    /// Legacy: submit pre-expanded candidate limbs.
    pub fn submit_batch(
        &mut self,
        limbs: &[u32],
        pattern: &[u64],
        limb_count: u16,
        num_candidates: u32,
    ) -> io::Result<GpuBatchResult> {
        self.batch_id = self.batch_id.wrapping_add(1);

        self.send_buf.clear();
        self.send_buf.extend_from_slice(&self.batch_id.to_le_bytes());
        self.send_buf.push(pattern.len() as u8);
        for &off in pattern {
            self.send_buf.extend_from_slice(&off.to_le_bytes());
        }
        self.send_buf.extend_from_slice(&limb_count.to_le_bytes());
        self.send_buf
            .extend_from_slice(&num_candidates.to_le_bytes());

        let limbs_bytes: &[u8] = bytemuck::cast_slice(limbs);
        self.send_buf.extend_from_slice(limbs_bytes);

        self.send_and_recv()
    }

    /// Compact: submit (primorial, first_candidate, f_values) — GPU reconstructs candidates.
    pub fn submit_compact_batch(
        &mut self,
        primorial_limbs: &[u32],
        first_cand_limbs: &[u32],
        f_values: &[u32],
        pattern: &[u64],
        limb_count: u16,
    ) -> io::Result<GpuBatchResult> {
        self.batch_id = self.batch_id.wrapping_add(1);
        let lc = limb_count as usize;

        self.send_buf.clear();
        self.send_buf.push(TAG_COMPACT);
        self.send_buf.extend_from_slice(&self.batch_id.to_le_bytes());
        self.send_buf.push(pattern.len() as u8);
        for &off in pattern {
            self.send_buf.extend_from_slice(&off.to_le_bytes());
        }
        self.send_buf.extend_from_slice(&limb_count.to_le_bytes());

        // Primorial limbs (padded to limb_count)
        for i in 0..lc {
            let v = primorial_limbs.get(i).copied().unwrap_or(0);
            self.send_buf.extend_from_slice(&v.to_le_bytes());
        }
        // First candidate limbs (padded to limb_count)
        for i in 0..lc {
            let v = first_cand_limbs.get(i).copied().unwrap_or(0);
            self.send_buf.extend_from_slice(&v.to_le_bytes());
        }

        self.send_buf
            .extend_from_slice(&(f_values.len() as u32).to_le_bytes());
        for &f in f_values {
            self.send_buf.extend_from_slice(&f.to_le_bytes());
        }

        self.send_and_recv()
    }

    fn send_and_recv(&mut self) -> io::Result<GpuBatchResult> {
        // Send length-prefixed message
        let len = self.send_buf.len() as u32;
        match &mut self.transport {
            Transport::Unix { writer, .. } => {
                writer.write_all(&len.to_le_bytes())?;
                writer.write_all(&self.send_buf)?;
                writer.flush()?;
            }
            Transport::Tcp { writer, .. } => {
                writer.write_all(&len.to_le_bytes())?;
                writer.write_all(&self.send_buf)?;
                writer.flush()?;
            }
        }

        // Read response
        let mut len_bytes = [0u8; 4];
        match &mut self.transport {
            Transport::Unix { reader, .. } => reader.read_exact(&mut len_bytes)?,
            Transport::Tcp { reader, .. } => reader.read_exact(&mut len_bytes)?,
        }
        let resp_len = u32::from_le_bytes(len_bytes) as usize;
        self.recv_buf.clear();
        self.recv_buf.resize(resp_len, 0);
        match &mut self.transport {
            Transport::Unix { reader, .. } => reader.read_exact(&mut self.recv_buf)?,
            Transport::Tcp { reader, .. } => reader.read_exact(&mut self.recv_buf)?,
        }

        // Decode BatchResult
        if self.recv_buf.len() < 8 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "response too short",
            ));
        }

        let mut pos = 4; // skip batch_id
        let num_rounds =
            u32::from_le_bytes(self.recv_buf[pos..pos + 4].try_into().unwrap()) as usize;
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
