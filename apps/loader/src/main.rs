#![feature(asm_const)]
#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[cfg(feature = "axstd")]
use axstd::println;
#[cfg(feature = "axstd")]
use axstd::process::exit;

const PLASH_START: usize = 0x22000000;

const SYS_HELLO: usize = 1;
const SYS_PUTCHAR: usize = 2;
const SYS_TERMINATE: usize = 3;

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

type BinNumType = u32;
const BIN_NUM_TYPE_BYTES: usize = 4;
type BinSizeType = u32;
const BIN_SIZE_TYPE_BYTES: usize = 4;

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    let apps_num = unsafe {
        (*(PLASH_START as *const BinNumType)).to_be()
    };
    
    println!("apps_num : {}", apps_num);

    // let start = PLASH_START as *const u8;
    let mut size_offset = PLASH_START + BIN_NUM_TYPE_BYTES;
    let mut start_offset = size_offset + BIN_SIZE_TYPE_BYTES * apps_num as usize;
    for i in 0 .. apps_num {
        println!("Load payload {} ...", i + 1);
        
        let load_size = unsafe {
            (*(size_offset as *const BinSizeType)).to_be() as usize
        };
        let load_code = unsafe {
            core::slice::from_raw_parts(start_offset as *const u8, load_size)
        };
        println!("load code {:?}; address [{:?}]", load_code, load_code.as_ptr());
        
        // app running aspace
        // SBI(0x80000000) -> App <- Kernel(0x80200000)
        // 0xffff_ffc0_0000_0000
        const RUN_START: usize = 0xffff_ffc0_8010_0000;

        let run_code = unsafe {
            core::slice::from_raw_parts_mut(RUN_START as *mut u8, load_size)
        };
        run_code.copy_from_slice(load_code);
        println!("run code {:?}; address [{:?}]", run_code, run_code.as_ptr());

        println!("Load payload {} ok!", i + 1);

        register_abi(SYS_HELLO, abi_hello as usize);
        register_abi(SYS_PUTCHAR, abi_putchar as usize);
        register_abi(SYS_TERMINATE, abi_terminate as usize);
    
        println!("Execute app ...");
        let arg0: u8 = b'A';
    
        // execute app
        unsafe { core::arch::asm!("
            li      t0, {abi_num}
            slli    t0, t0, 3
            la      t1, {abi_table}
            add     t1, t1, t0
            ld      t1, (t1)
            jalr    t1
            li      t0, {abi_ter}
            slli    t0, t0, 3
            la      t1, {abi_table}
            add     t1, t1, t0
            ld      t1, (t1)
            jalr    t1
            li      t2, {run_start}
            jalr    t2",
            run_start = const RUN_START,
            abi_table = sym ABI_TABLE,
            //abi_num = const SYS_HELLO,
            abi_num = const SYS_PUTCHAR,
            abi_ter = const SYS_TERMINATE,
            in("a0") arg0,
        )}

        size_offset += BIN_SIZE_TYPE_BYTES;
        start_offset += load_size;
    }
}