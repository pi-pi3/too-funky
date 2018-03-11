
#[derive(Clone, Default)]
pub struct Args {
    pub eax: usize,
    pub edx: usize,
    pub ecx: usize,
    pub ebx: usize,
    pub edi: usize,
    pub esi: usize,
    pub esp: usize,
    pub ebp: usize,
}

#[repr(C, packed)]
pub struct ErrArgs<E> {
    pub eip: usize,
    pub cs: usize,
    pub eflags: usize,
    pub err: E,
}

// this makes it impossible to call an interrupt handler from Rust
pub enum NoCall {}

#[macro_export]
macro_rules! inner_func {
    ($func:ident, $( $arg:ident : $type:ty, )* $body:block) => {
        unsafe fn $func ( $( $arg : $type ),* ) {
            $body
        }
    }
}

#[macro_export]
macro_rules! regs {
    ($regs:expr) => {
        asm!("": 
                 "={eax}"($regs.eax),
                 "={edx}"($regs.edx),
                 "={ecx}"($regs.ecx),
                 "={ebx}"($regs.ebx),
                 "={edi}"($regs.edi),
                 "={esi}"($regs.esi),
                 "={esp}"($regs.esp),
                 "={ebp}"($regs.ebp))
    }
}

#[macro_export]
macro_rules! pushad {
    () => { asm!( "pushad" : : : : "intel", "volatile") }
}

#[macro_export]
macro_rules! popad {
    () => { asm!( "popad" : : : : "intel", "volatile") }
}

#[macro_export]
macro_rules! iretd {
    () => { asm!( "iretd" : : : : "intel", "volatile") }
}

#[macro_export]
macro_rules! interrupt_handler {
    {
        pub unsafe extern fn $func:ident ( $( $arg:ident : $reg:ident ),* ) $body:block
    } => {
        #[naked]
        pub unsafe extern fn $func ( _: $crate::interrupt::macros::NoCall ) {
            inner_func!(inner, $( $arg : usize, )* $body);
            let mut regs = $crate::interrupt::macros::Args::default();
            regs!(regs);
            pushad!();
            inner($( regs.$reg ),*);
            popad!();
            iretd!();
        }
    }
}

#[macro_export]
macro_rules! exception_handler {
    {
        pub unsafe extern fn $func:ident < $err:ty > ( $( $errarg:ident : $errreg:ident ),* ; $( $arg:ident : $reg:ident ),* ) $body:block
    } => {
        #[naked]
        pub unsafe extern fn $func ( args: ErrArgs<$err>, _: $crate::interrupt::macros::NoCall ) {
            inner_func!(inner, $( $errarg : usize, )* $( $arg : usize, )* $body);
            let mut regs = $crate::interrupt::macros::Args::default();
            regs!(regs);
            pushad!();
            inner($( args.$errreg, )* $( regs.$reg),*);
            popad!();
            iretd!();
        }
    }
}
