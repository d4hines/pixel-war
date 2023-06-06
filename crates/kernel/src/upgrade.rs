// Adapted from  https://gitlab.com/tezos/tezos/-/blob/f8cf9f252990caa2f2d366564ce9e8d29724bafc/src/kernel_sdk/installer-kernel/src/lib.rs
// Original license header:

// SPDX-FileCopyrightText: 2023 TriliTech <contact@trili.tech>
//
// SPDX-License-Identifier: MIT

//! Installer kernel for tezos smart rollups.
//!
//! # About
//!
//! When originating a smart rollup, you must supply a kernel - the program to be executed
//! by the rollup. This origination kernel must fit within the size of a Layer-1 operation
//! (about 32KB).
//!
//! Almost all useful kernels are larger than this, however. As a result, it is recommended
//! to use this installer kernel. When originating a rollup, you may use a configured
//! installer kernel - which will then proceed to upgrade to your desired kernel.

#![forbid(unsafe_code)]
use tezos_smart_rollup::core_unsafe::MAX_FILE_CHUNK_SIZE;
use tezos_smart_rollup::core_unsafe::PREIMAGE_HASH_SIZE;
use tezos_smart_rollup::dac::reveal_loop;
use tezos_smart_rollup::dac::V0SliceContentPage;
use tezos_smart_rollup::dac::MAX_PAGE_SIZE;
use tezos_smart_rollup::prelude::*;
use tezos_smart_rollup::storage::path::RefPath;

// Path of currently running kernel.
const KERNEL_BOOT_PATH: RefPath = RefPath::assert_from(b"/kernel/boot.wasm");

// Path that we write the kernel to, before upgrading.
const PREPARE_KERNEL_PATH: RefPath = RefPath::assert_from(b"/installer/kernel/boot.wasm");

// Support 3 levels of hashes pages, and then bottom layer of content.
const MAX_DAC_LEVELS: usize = 4;

pub fn install_kernel<Host: Runtime>(
    host: &mut Host,
    root_hash: &[u8; PREIMAGE_HASH_SIZE],
) -> Result<(), &'static str> {
    let mut buffer = [0; MAX_PAGE_SIZE * MAX_DAC_LEVELS];

    let mut write_kernel_page = write_kernel_page();

    reveal_loop(
        host,
        0,
        root_hash,
        buffer.as_mut_slice(),
        MAX_DAC_LEVELS,
        &mut write_kernel_page,
    )?;

    Runtime::store_move(host, &PREPARE_KERNEL_PATH, &KERNEL_BOOT_PATH)
        .map_err(|_| "FAILED to install kernel in KERNEL_PATH")?;

    Ok(())
}

fn write_kernel_page<Host: Runtime>(
) -> impl FnMut(&mut Host, V0SliceContentPage) -> Result<(), &'static str> {
    let mut kernel_size = 0;
    move |host, page| {
        let written = append_content(host, kernel_size, page)?;
        kernel_size += written;
        Ok(())
    }
}

/// Appends the content of the page path given.
fn append_content<Host: Runtime>(
    host: &mut Host,
    kernel_size: usize,
    content: V0SliceContentPage,
) -> Result<usize, &'static str> {
    let content = content.as_ref();

    let mut size_written = 0;
    while size_written < content.len() {
        let num_to_write = usize::min(MAX_FILE_CHUNK_SIZE, content.len() - size_written);
        let bytes_to_write = &content[size_written..(size_written + num_to_write)];

        Runtime::store_write(
            host,
            &PREPARE_KERNEL_PATH,
            bytes_to_write,
            kernel_size + size_written,
        )
        .map_err(|_| "Failed to write kernel content page")?;

        size_written += num_to_write;
    }

    Ok(size_written)
}
