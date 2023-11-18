#![feature(asm_const)]
#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[cfg(feature = "axstd")]
use axstd::println;
#[cfg(feature = "axstd")]
use axstd::process::exit;

const PLASH_START: usize = 0x22000000;

const RUN_START: usize = 0x4010_0000;

const SYS_HELLO: usize = 1;
const SYS_PUTCHAR: usize = 2;
const SYS_TERMINATE: usize = 3;
const SYS_PRINT: usize = 4;

static mut ABI_TABLE: [usize; 16] = [0; 16];

fn register_abi(num: usize, handle: usize) {
    unsafe { ABI_TABLE[num] = handle; }
}

fn abi_hello() {
    println!("[ABI:Hello] Hello, Apps!");
}

fn abi_putchar(c: char) {
    println!("[ABI:Print] {c}");
}

fn abi_terminate() {
    println!("[ABI:Terminate] ArceOS Terminate");
    exit(0);
}

fn abi_print(x: u64) {
    println!("[ABI:P] {:#x}", x);
}

type BinNumType = u32;
const BIN_NUM_TYPE_BYTES: usize = 4;
type BinSizeType = u32;
const BIN_SIZE_TYPE_BYTES: usize = 4;

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    unsafe { init_app_page_table(); }
    
    register_abi(SYS_HELLO, abi_hello as usize);
    register_abi(SYS_PUTCHAR, abi_putchar as usize);
    register_abi(SYS_TERMINATE, abi_terminate as usize);
    register_abi(SYS_PRINT, abi_print as usize);

    let apps_num = unsafe {
        (*(PLASH_START as *const BinNumType)).to_be()
    };
    
    println!("apps_num : {}", apps_num);

    // let start = PLASH_START as *const u8;
    let mut size_offset = PLASH_START + BIN_NUM_TYPE_BYTES;
    let mut start_offset = size_offset + BIN_SIZE_TYPE_BYTES * apps_num as usize;
    for i in 0 .. apps_num {
        unsafe { switch_app_aspace(i as usize); }
        println!("Load payload {} ...", i + 1);
        
        let load_size = unsafe {
            (*(size_offset as *const BinSizeType)).to_be() as usize
        };
        let load_code = unsafe {
            core::slice::from_raw_parts(start_offset as *const u8, load_size)
        };
        println!("load code {:?}; address [{:?}]", load_code, load_code.as_ptr());

        // switch aspace from kernel to app

        let run_code = unsafe {
            core::slice::from_raw_parts_mut(RUN_START as *mut u8, load_size)
        };
        run_code.copy_from_slice(load_code);
        println!("run code {:?}; address [{:?}] size [{}]", run_code, run_code.as_ptr(), load_size);

        println!("Load payload {} ok!", i + 1);
    
        println!("Execute app ...");
        // execute app
        unsafe { core::arch::asm!("
            la      a0, {abi_table}
            li      t2, {run_start}
            jalr    t2",
            run_start = const RUN_START,
            abi_table = sym ABI_TABLE,
        )}

        size_offset += BIN_SIZE_TYPE_BYTES;
        start_offset += load_size;
    }
}

#[link_section = ".data.app_page_table"]
static mut APP_PT_SV39: [u64; 512] = [0; 512];

unsafe fn init_app_page_table() {
    // 0x8000_0000..0xc000_0000, VRWX_GAD, 1G block
    APP_PT_SV39[2] = (0x80000 << 10) | 0xef;
    // 0xffff_ffc0_8000_0000..0xffff_ffc0_c000_0000, VRWX_GAD, 1G block
    APP_PT_SV39[0x102] = (0x80000 << 10) | 0xef;

    // 0x0000_0000..0x4000_0000, VRWX_GAD, 1G block
    APP_PT_SV39[0] = (0x00000 << 10) | 0xef;

    // For App aspace!
    // 0x4000_0000..0x8000_0000, VRWX_GAD, 1G block
    APP_PT_SV39[1] = (0x80000 << 10) | 0xef;
}

unsafe fn switch_app_aspace(asid: usize) {
    use riscv::register::satp;
    let page_table_root = APP_PT_SV39.as_ptr() as usize - axconfig::PHYS_VIRT_OFFSET;
    satp::set(satp::Mode::Sv39, asid, page_table_root >> 12);
    riscv::asm::sfence_vma_all();
}