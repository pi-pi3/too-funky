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
#[doc(hidden)]
macro_rules! inner_func {
    (
        $func:ident,
        $( $errarg:ident : $errreg:ident ),* ;
        $( $arg:ident : $reg:ident, )*
        $body:block
    ) => {
        unsafe fn $func ( $( $errarg : usize ),* ) {
            let mut regs = $crate::arch::interrupt::macros::Args::default();
            regs!(regs);

            $(
                let $arg = regs.$reg;
            )*

            $body
        }
    }
}

#[macro_export]
#[doc(hidden)]
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
#[doc(hidden)]
macro_rules! pushad {
    () => { asm!( "pushal" : : : : "volatile") }
}

#[macro_export]
#[doc(hidden)]
macro_rules! popad {
    () => { asm!( "popal" : : : : "volatile") }
}

#[macro_export]
#[doc(hidden)]
macro_rules! iretd {
    () => { asm!( "iretl" : : : : "volatile") }
}

#[macro_export]
macro_rules! interrupt_handler {
    {
        pub unsafe extern fn $func:ident ( $( $arg:ident : $reg:ident ),* )
            $body:block
    } => {
        #[naked]
        pub unsafe extern fn $func ( _: $crate::arch::interrupt::macros::NoCall ) {
            pushad!();
            inner_func!(inner, ; $( $arg : $reg, )* $body);
            inner();
            popad!();
            iretd!();
            unreachable!();
        }
    }
}

#[macro_export]
macro_rules! exception_handler {
    {
        pub unsafe extern fn $func:ident < $err:ty > (
            $( $errarg:ident : $errreg:ident ),* ;
            $( $arg:ident : $reg:ident ),*
        ) $body:block
    } => {
        #[naked]
        #[allow(unused_variables)]
        pub unsafe extern fn $func (
            args: $crate::arch::interrupt::macros::ErrArgs<$err>,
            _: $crate::arch::interrupt::macros::NoCall
        ) {
            pushad!();
            inner_func!(
                inner,
                $( $errarg : $errreg ),* ;
                $( $arg : $reg, )*
                $body
            );
            inner($( args.$errarg ),*);
            popad!();
            iretd!();
            unreachable!();
        }
    }
}

#[macro_export]
macro_rules! interrupt_handlers {
    {} => {};
    {
        pub unsafe extern fn $func:ident ( $( $arg:ident : $reg:ident ),* )
            $body:block
        $( $rest:tt )*
    } => {
        interrupt_handler! {
            pub unsafe extern fn $func ( $( $arg : $reg ),* ) $body
        }
        interrupt_handlers! { $( $rest )* }
    };
    {
        pub unsafe extern fn $func:ident < $err:ty > (
            $( $errarg:ident : $errreg:ident ),* ;
            $( $arg:ident : $reg:ident ),*
        ) $body:block
        $( $rest:tt )*
    } => {
        exception_handler! {
            pub unsafe extern fn $func < $err > (
                $( $errarg : $errreg ),* ;
                $( $arg : $reg ),*
            ) $body
        }
        interrupt_handlers! { $( $rest )* }
    }
}
