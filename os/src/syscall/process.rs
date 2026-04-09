//! Process management syscalls

use core::slice::from_raw_parts;
use crate::config::PAGE_SIZE;
use crate::mm::{MapPermission, PageTable, VirtAddr, translated_byte_buffer, PTEFlags};
use crate::task::{change_program_brk, current_user_token, exit_current_and_run_next, mmap_current, munmap_current, suspend_current_and_run_next, syscall_count};
use crate::timer::get_time_us;
#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    let time_val = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    let token = current_user_token();
    let len = core::mem::size_of::<TimeVal>();
    let buffer = translated_byte_buffer(token, ts as *const u8, len);
    let bytes = unsafe {
        from_raw_parts(&time_val as *const TimeVal as *const u8, len)
    };
    let mut start = 0;
    for buf in buffer {
        let end = start + buf.len();
        buf.copy_from_slice(&bytes[start..end]);
        start = end;
    }
    if start != bytes.len() {
        return -1;
    }
    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");
    let token = current_user_token();
    let page_table = PageTable::from_token(token);
    match trace_request {
        0 => {
            
            let va = VirtAddr::from(id);
            if let Some(pte) = page_table.translate(va.floor()){
                let f = pte.flags();
                if pte.is_valid() && f.contains(PTEFlags::U) && f.contains(PTEFlags::R){
                    let ppn = pte.ppn();
                    let value = ppn.get_bytes_array()[va.page_offset()];
                    return value as isize;
                }
            }
            -1
        },
        1 => {
            let va = VirtAddr::from(id);
            if let Some(pte) = page_table.translate(va.floor()){
                let f = pte.flags();
                if pte.is_valid() && f.contains(PTEFlags::U) && f.contains(PTEFlags::W){
                    let ppn = pte.ppn();
                    ppn.get_bytes_array()[va.page_offset()] = data as u8;
                
                    return 0;
                }
            }
            -1
        },
        2 => {
            if id < 512 {
                return syscall_count(id) as isize;
            }
            -1
        },
        _ =>{
            -1
        }
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    if start % PAGE_SIZE != 0 || port & !0x7 != 0 || port & 0x7 == 0 {
        return -1
    }
    let len = (len + PAGE_SIZE - 1) / PAGE_SIZE * PAGE_SIZE;
    let start_va = VirtAddr::from(start);
    let end_va = VirtAddr::from(start + len);
    let mut perm = MapPermission::U;
    if port & 1 != 0 {
        perm |= MapPermission::R; 
    }
    if port & 2 != 0 {
        perm |= MapPermission::W;
    }
    if port & 4 != 0 {
        perm |= MapPermission::X;
    }
    if mmap_current(start_va, end_va, perm) {
        return 0
    }
    -1
    
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    if start % PAGE_SIZE != 0 {
        return -1
    }
    let len = (len + PAGE_SIZE - 1) / PAGE_SIZE * PAGE_SIZE;
    let start_va = VirtAddr::from(start);
    let end_va = VirtAddr::from(start + len);
    
    if munmap_current(start_va, end_va) {
        return 0
    }
    -1
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
