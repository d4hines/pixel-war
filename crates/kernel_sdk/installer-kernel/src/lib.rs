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

#![cfg_attr(all(target_arch = "wasm32", not(feature = "std")), no_std)]
#![forbid(unsafe_code)]

mod instr;
mod preimage;

use core::panic::PanicInfo;
use instr::read_config_program_size;
use tezos_smart_rollup::host::Runtime;
use tezos_smart_rollup::storage::path::RefPath;
use tezos_smart_rollup_installer_config::binary::{
    completed, read_size, EncodingSize, NomReader, RefConfigInstruction,
};

use crate::instr::{handle_instruction, read_instruction_bytes};

// Path of currently running kernel.
const KERNEL_BOOT_PATH: RefPath = RefPath::assert_from(b"/kernel/boot.wasm");

// Installer kernel will copy to this path before execution of config.
// This is done in order avoid rewriting kernel during config execution.
const AUXILIARY_KERNEL_BOOT_PATH: RefPath =
    RefPath::assert_from(b"/__installer_kernel/auxiliary/kernel/boot.wasm");

// Support 3 levels of hashes pages, and then bottom layer of content.
const MAX_DAC_LEVELS: usize = 4;

#[cfg(all(feature = "entrypoint", target_arch = "wasm32"))]
tezos_smart_rollup::kernel_entry!(installer);

/// Installer.
pub fn installer<Host: Runtime>(host: &mut Host) {
    if let Ok(config_program_size) = read_config_program_size(host) {
        if let Err(e) = install_kernel(host, config_program_size as usize) {
            Runtime::write_debug(host, e)
        }
    } else {
        host.write_debug("Failed to read size of config program")
    }
}

/// Panic handler used when targetting wasm.
///
/// Any error when installing a kernel is fatal, therefore we
/// handle panics by panicking again, which aborts execution.
#[cfg_attr(all(target_arch = "wasm32", not(feature = "std")), panic_handler)]
#[allow(dead_code)]
fn panic(_info: &PanicInfo) -> ! {
    panic!()
}

fn install_kernel(
    host: &mut impl Runtime,
    config_program_size: usize,
) -> Result<(), &'static str> {
    let mut config_instruction_buffer = [0; RefConfigInstruction::MAX_SIZE];

    let kernel_size = host
        .store_value_size(&KERNEL_BOOT_PATH)
        .map_err(|_| "Failed to read kernel boot path size")?;

    host.store_copy(&KERNEL_BOOT_PATH, &AUXILIARY_KERNEL_BOOT_PATH)
        .map_err(|_| "Failed to copy kernel boot before config execution")?;

    let end_offset = kernel_size - 4;
    let mut instr_offset = end_offset - config_program_size;
    while instr_offset < end_offset {
        let instr_size =
            read_size(host, &AUXILIARY_KERNEL_BOOT_PATH, &mut instr_offset)? as usize;
        read_instruction_bytes(
            host,
            &AUXILIARY_KERNEL_BOOT_PATH,
            &mut instr_offset,
            &mut config_instruction_buffer[..instr_size],
        )?;
        let instr =
            RefConfigInstruction::nom_read(&config_instruction_buffer[..instr_size])
                .map_err(|_| "Couldn't decode config instruction")
                .and_then(completed)?;
        handle_instruction(host, instr)?;
    }

    host.store_delete(&AUXILIARY_KERNEL_BOOT_PATH)
        .map_err(|_| "Failed to delete auxiliary kernel boot after config execution")?;

    Ok(())
}
