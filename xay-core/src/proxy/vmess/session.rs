use bytes::{Buf, BufMut, BytesMut};
use rand::{distributions::Standard, Rng};
use std::hash::{BuildHasher, Hasher};

use crate::common::protocol::RequestHeader;

use super::account::MemoryAccount;

const VERSION: u8 = 1;

pub struct ClientSession<Hash> {
    is_aead: bool,
    req_body_key: [u8; 16],
    req_body_iv: [u8; 16],
    res_body_key: [u8; 16],
    res_body_iv: [u8; 16],
    id_hash: fn(&[u8]) -> Hash,
    res_header: u8,
}

impl<Hash> ClientSession<Hash> {
    pub fn encode_req_header<B>(
        &self,
        header: &RequestHeader<MemoryAccount>,
        b: &mut B,
    ) -> Result<(), anyhow::Error>
    where
        B: BufMut,
        Hash: Hasher,
    {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        let timestamp = rand::thread_rng().gen_range(now..now + 2 * 30) - 30;
        let account = &header.user.account;

        if !self.is_aead {
            let mut id_hash = (self.id_hash)(&account.any_valid_id().uuid);
            id_hash.write_u64(timestamp);
            b.put_u64(id_hash.finish())
        }

        let mut buf = BytesMut::new();

        buf.reserve(35);

        buf.put_u8(VERSION);
        buf.put_slice(&self.req_body_iv);
        buf.put_slice(&self.req_body_key);
        buf.put_u8(self.res_header);
        buf.put_u8(header.option);

        let padding_len = rand::thread_rng().gen_range(0..16);
        let security = (padding_len << 4 | header.security) as u8;

        buf.put_slice(&[security, 0, header.req_cmd]);

        // TODO: write port

        if padding_len > 0 {
            let v: Vec<u8> = rand::thread_rng()
                .sample_iter(Standard)
                .take(padding_len as usize)
                .collect();

            buf.put_slice(v.as_slice());
        }

        {
            let mut fnv1a = fnv::FnvBuildHasher::default().build_hasher();
            fnv1a.write(buf.chunk());
            buf.put_u64(fnv1a.finish());
        }

        if !self.is_aead {}

        Ok(())
    }
}
