//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! The Global Descriptor Table (GDT) is used for configuring segmentation.
//!
//! As we use paging rather than segmentation for memory management, we do
//! not actually use the GDT, but some x86 functionality still require it
//! to be properly configured.
use arch::cpu::segment;

const GDT_SIZE: usize = 512;

type Gdt = [segment::Descriptor; GDT_SIZE];
