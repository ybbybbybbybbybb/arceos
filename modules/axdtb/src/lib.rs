#![no_std]

extern crate alloc;
use alloc::vec::Vec;
use fdt::{Fdt, FdtError};
// use axdtb::util::SliceRead;
// 参考类型定义
pub struct DtbInfo {
    pub memory_addr: usize,
    pub memory_size: usize,
    pub mmio_regions: Vec<(usize, usize)>,
}

pub fn parse_dtb(dtb_pa: usize) -> Result<DtbInfo, FdtError> {
    
    let fdt = unsafe {Fdt::from_ptr(dtb_pa as *const u8)? };

    if let Some(mem_region) = fdt.memory().regions().next() {
        let memory_addr = mem_region.starting_address as usize;
        if mem_region.size.is_none() {
            return Err(FdtError::BadPtr);
        }
        let memory_size = mem_region.starting_address as usize + mem_region.size.unwrap() as usize;
        let mut mmio_regions = Vec::new();
        for node in fdt.find_all_nodes("/soc/virtio_mmio") {
            if node.reg().is_some() {
                if let Some(mem_region) = node.reg().unwrap().next() {
                    if let Some(size) = mem_region.size {
                       mmio_regions.push((mem_region.starting_address as usize, size));
                    }
                }
            }
        }
        
        return Ok(DtbInfo {
            memory_addr,
            memory_size,
            mmio_regions,
        })
    }
    Err(FdtError::BadPtr)
}