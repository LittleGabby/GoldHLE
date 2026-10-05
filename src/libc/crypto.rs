/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![allow(non_upper_case_globals)]
//! CommonCrypto and friends

use crate::cpu::Cpu;
use crate::dyld::FunctionExports;
use crate::mem::{ConstVoidPtr, GuestUSize, MutPtr, MutVoidPtr, Ptr};
use crate::{export_c_func, Environment};
use aes::cipher::{BlockDecrypt, BlockEncrypt, KeyInit};
use aes::{Aes128, Aes192, Aes256};
use digest::Digest;
use md5::Md5;
use sha1::Sha1;

fn CC_MD5(env: &mut Environment, data: ConstVoidPtr, len: u32, md: MutPtr<u8>) -> MutPtr<u8> {
    let mut hasher = Md5::new();
    hasher.update(env.mem.bytes_at(data.cast(), len));
    let digest = hasher.finalize();
    env.mem.bytes_at_mut(md, 16).copy_from_slice(&digest[..]);
    md
}

fn CC_SHA1(env: &mut Environment, data: ConstVoidPtr, len: u32, md: MutPtr<u8>) -> MutPtr<u8> {
    let mut hasher = Sha1::new();
    hasher.update(env.mem.bytes_at(data.cast(), len));
    let digest = hasher.finalize();
    env.mem.bytes_at_mut(md, 20).copy_from_slice(&digest[..]);
    md
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(CC_MD5(_, _, _)),
    export_c_func!(CC_SHA1(_, _, _)),
    // CCCrypt has 11 parameters, but the host-call ABI only supports up to 10
    // (including the implicit env parameter). The final parameter,
    // dataOutMoved, is therefore read directly from the guest stack, which is
    // valid for the duration of the host call.
    export_c_func!(CCCrypt(_, _, _, _, _, _, _, _, _, _)),
];

// CCCryptorStatus
const kCCSuccess: i32 = 0;
const kCCParamError: i32 = -4300;
const kCCBufferTooSmall: i32 = -4301;
const kCCDecodeError: i32 = -4304;
const kCCAlignmentError: i32 = -4303;

// CCOperation
const kCCEncrypt: u32 = 0;
const kCCDecrypt: u32 = 1;
// CCAlgorithm
const kCCAlgorithmAES: u32 = 0;
const kCCAlgorithmRC4: u32 = 4;
// CCOptions
const kCCOptionPKCS7Padding: u32 = 0x0001;
const kCCOptionECBMode: u32 = 0x0002;

const AES_BLOCK_SIZE: usize = 16;

fn aes_block_crypt(key: &[u8], op: u32, block: &mut [u8; AES_BLOCK_SIZE]) {
    let generic_block = aes::Block::from_mut_slice(block);
    if op == kCCEncrypt {
        if key.len() == 16 {
            Aes128::new_from_slice(key)
                .unwrap()
                .encrypt_block(generic_block);
        } else if key.len() == 24 {
            Aes192::new_from_slice(key)
                .unwrap()
                .encrypt_block(generic_block);
        } else {
            Aes256::new_from_slice(key)
                .unwrap()
                .encrypt_block(generic_block);
        }
    } else if key.len() == 16 {
        Aes128::new_from_slice(key)
            .unwrap()
            .decrypt_block(generic_block);
    } else if key.len() == 24 {
        Aes192::new_from_slice(key)
            .unwrap()
            .decrypt_block(generic_block);
    } else {
        Aes256::new_from_slice(key)
            .unwrap()
            .decrypt_block(generic_block);
    }
}

#[allow(non_snake_case, clippy::too_many_arguments)]
fn CCCrypt(
    env: &mut Environment,
    op: u32,
    alg: u32,
    options: u32,
    key: ConstVoidPtr,
    key_length: GuestUSize,
    iv: ConstVoidPtr,
    data_in: ConstVoidPtr,
    data_in_length: GuestUSize,
    data_out: MutVoidPtr,
    data_out_available: GuestUSize,
) -> i32 {
    // The eleventh parameter (dataOutMoved) is on the guest stack, one slot
    // past the tenth. See the note on the FUNCTIONS export above.
    let stack_ptr = Ptr::<u32, false>::from_bits(env.cpu.regs()[Cpu::SP]);
    let data_out_moved: MutPtr<u32> = Ptr::from_bits(env.mem.read(stack_ptr + 6));

    if op != kCCEncrypt && op != kCCDecrypt {
        return kCCParamError;
    }
    if key.is_null() || data_in.is_null() || data_out.is_null() {
        return kCCParamError;
    }
    if data_in_length == 0 {
        env.mem.write(data_out_moved, 0u32);
        return kCCSuccess;
    }
    if data_out_available < data_in_length {
        return kCCBufferTooSmall;
    }

    if alg == kCCAlgorithmAES {
        if !matches!(key_length, 16 | 24 | 32) {
            return kCCParamError;
        }
        let in_bytes = env.mem.bytes_at(data_in.cast(), data_in_length).to_vec();
        let key_bytes = env.mem.bytes_at(key.cast(), key_length).to_vec();
        let mut chain = if iv.is_null() {
            vec![0u8; AES_BLOCK_SIZE]
        } else {
            env.mem.bytes_at(iv.cast(), AES_BLOCK_SIZE as u32).to_vec()
        };

        // When no IV is provided and ECB mode isn't requested, Apple treats
        // this as CBC mode with a zero IV.
        let ecb = (options & kCCOptionECBMode) != 0;

        // Block ciphers require input aligned to the block size, unless
        // padding is requested on the encrypt path.
        let padding_requested = (options & kCCOptionPKCS7Padding) != 0;
        if !in_bytes.len().is_multiple_of(AES_BLOCK_SIZE)
            && (op == kCCDecrypt || !padding_requested)
        {
            return kCCAlignmentError;
        }
        // PKCS7 always adds at least one byte; when the input length is an
        // exact multiple of the block size, a whole block of padding is added.
        let padded_len = if (options & kCCOptionPKCS7Padding) != 0 && op == kCCEncrypt {
            in_bytes.len() + (AES_BLOCK_SIZE - (in_bytes.len() % AES_BLOCK_SIZE))
        } else {
            in_bytes.len()
        };
        if padded_len as u32 > data_out_available {
            return kCCBufferTooSmall;
        }
        let mut work = in_bytes.clone();
        if op == kCCEncrypt && padded_len > work.len() {
            let pad = (padded_len - work.len()) as u8;
            work.resize(padded_len, pad);
        }
        let mut out_bytes = vec![0u8; padded_len];
        for (i, chunk) in work.chunks(AES_BLOCK_SIZE).enumerate() {
            let mut block = [0u8; AES_BLOCK_SIZE];
            block[..chunk.len()].copy_from_slice(chunk);
            let orig = block;
            // CBC: on encrypt the chaining happens before the cipher
            // (c = E(p ^ chain)); on decrypt it happens after
            // (p = D(c) ^ chain).
            if !ecb && op == kCCEncrypt {
                for j in 0..AES_BLOCK_SIZE {
                    block[j] ^= chain[j];
                }
            }
            match key_length {
                16 => aes_block_crypt(&key_bytes, op, &mut block),
                24 => aes_block_crypt(&key_bytes, op, &mut block),
                32 => aes_block_crypt(&key_bytes, op, &mut block),
                _ => unreachable!(),
            }
            if !ecb {
                if op == kCCEncrypt {
                    chain = block.to_vec();
                } else {
                    for j in 0..AES_BLOCK_SIZE {
                        block[j] ^= chain[j];
                    }
                    chain = orig.to_vec();
                }
            }
            let start = i * AES_BLOCK_SIZE;
            let end = (start + AES_BLOCK_SIZE).min(out_bytes.len());
            out_bytes[start..end].copy_from_slice(&block[..end - start]);
        }

        let mut produced = out_bytes.len();
        if (options & kCCOptionPKCS7Padding) != 0 && op == kCCDecrypt {
            match out_bytes.last() {
                Some(&pad)
                    if pad >= 1
                        && (pad as usize) <= AES_BLOCK_SIZE
                        && out_bytes[out_bytes.len() - pad as usize..]
                            .iter()
                            .all(|&b| b == pad) =>
                {
                    produced -= pad as usize;
                }
                _ => {
                    log!("CCCrypt: invalid PKCS7 padding, returning kCCDecodeError");
                    return kCCDecodeError;
                }
            }
        }
        if produced as u32 > data_out_available {
            return kCCBufferTooSmall;
        }
        env.mem
            .bytes_at_mut(data_out.cast(), produced as u32)
            .copy_from_slice(&out_bytes[..produced]);
        if op == kCCDecrypt {
            let key_addr_bits = key.to_bits();
            let dump_path = format!("/tmp/ccdump/{:016x}_{}.bin", key_addr_bits, produced);
            let _ = std::fs::write(&dump_path, &out_bytes[..produced]);
        }
        env.mem.write(data_out_moved, produced as u32);
        kCCSuccess
    } else if alg == kCCAlgorithmRC4 {
        if key.is_null() || key_length == 0 {
            return kCCParamError;
        }
        let in_bytes = env.mem.bytes_at(data_in.cast(), data_in_length).to_vec();
        let key_bytes = env.mem.bytes_at(key.cast(), key_length).to_vec();
        let mut s: Vec<u8> = (0..=255u8).collect();
        let mut j: usize = 0;
        for i in 0..256usize {
            j = (j + s[i] as usize + key_bytes[i % key_bytes.len()] as usize) % 256;
            s.swap(i, j);
        }
        let (mut i, mut j): (usize, usize) = (0, 0);
        let mut out_bytes = vec![0u8; in_bytes.len()];
        for (idx, &byte) in in_bytes.iter().enumerate() {
            i = (i + 1) % 256;
            j = (j + s[i] as usize) % 256;
            s.swap(i, j);
            out_bytes[idx] = byte ^ s[(s[i] as usize + s[j] as usize) % 256];
        }
        env.mem
            .bytes_at_mut(data_out.cast(), data_in_length)
            .copy_from_slice(&out_bytes);
        env.mem.write(data_out_moved, data_in_length);
        kCCSuccess
    } else {
        log!("CCCrypt: unimplemented algorithm {}", alg);
        kCCParamError
    }
}
