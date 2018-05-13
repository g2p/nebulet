use x86_64::registers::model_specific::Msr;
use x86_64::registers::flags::Flags;
use core::ptr;

use arch::interrupt;
use arch::asm::read_gs_offset64;

use task::{Thread, thread::State, scheduler::Scheduler};
// use spin::{RwLock, Once};

use alloc::boxed::Box;
// use alloc::Vec;

// static GLOBAL: Once<Global> = Once::new();

pub type CpuId = u32;

pub struct Cpu {
    /// The cpu id (starts at 0)
    cpu_id: CpuId,
}

impl Cpu {
    pub fn id(&self) -> CpuId {
        self.cpu_id
    }
}

pub struct IrqController;

impl IrqController {
    #[inline]
    pub unsafe fn disable() {
        interrupt::disable();
    }

    #[inline]
    pub unsafe fn enable() {
        interrupt::enable();
    }

    #[inline]
    #[must_use]
    pub fn enabled() -> bool {
        let rflags: Flags;
        unsafe {
            asm!("pushfq; pop $0" : "=r"(rflags) : : "memory" : "intel", "volatile");
        }
        rflags.contains(Flags::IF)
    }
}

pub unsafe fn init(cpu_id: u32) {
    let cpu = Box::new(Cpu {
        cpu_id,
    });

    let mut cpu_local = Box::new(Local::new(Box::leak(cpu)));

    cpu_local.direct = &mut *cpu_local as *mut _;

    Msr::new(0xC0000101)
        .write(Box::leak(cpu_local) as *mut _ as u64);
}

// /// Global system data
// pub struct Global {
//     /// List of all locals that are currently online.
//     pub locals: RwLock<Vec<Local>>,
// }

// impl Global {
//     fn new() -> Global {
//         Global {
//             locals: RwLock::new(Vec::new()),
//         }
//     }
// }

/// Each cpu contains this in the gs register.
pub struct Local {
    direct: *mut Local,
    /// Reference to the current `Cpu`.
    pub cpu: &'static mut Cpu,
    /// The scheduler associated with this cpu.
    pub scheduler: Scheduler,
}

impl Local {
    fn new(cpu: &'static mut Cpu) -> Local {
        let idle_thread = Thread::new(4096, idle_thread_entry, 0)
            .unwrap();

        let mut kernel_thread = Thread::new(0, idle_thread_entry, 0)
            .unwrap();
            
        kernel_thread.state = State::Suspended;

        let scheduler = Scheduler::new(kernel_thread, idle_thread);

        Local {
            direct: ptr::null_mut(),
            cpu,
            scheduler,
        }
    }

    pub fn current() -> &'static mut Local {
        unsafe {
            &mut *(read_gs_offset64!(0x0) as *mut Local)
        }
    }
}

extern fn idle_thread_entry(_: usize) {
    loop {
        unsafe { ::arch::interrupt::halt(); }
    }
}