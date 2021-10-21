#![feature(c_variadic)]
#![feature(asm)]

extern crate libc;

fn main() {
    let mut args = [8, 8, 8, 8, 8];
    let _sum = unsafe { call_variadic(&mut args) };
    println!("{}", &_sum);
}

unsafe fn call_variadic(args: &mut [u64]) -> u64 {
    let result;

    let num_args = args.len();
    let args_ptr = args.as_mut_ptr();
    let extra_args = if num_args > 5 { num_args - 5 } else { 0 };

    // very ugly and wildly unsafe
    // VERY specific to the sysv x64 abi
    // There's probably a better way to do this and not clobber so many registers
    asm!(
        // align stack to 16
        "sub rsp, 8",
        "cmp r10, 5",
        // if num_args > 5
        "jg 6f",
        // == 5
        "je 7f",
        // == 2
        "cmp r10, 2",
        "je 3f",
        // == 3
        "cmp r10, 3",
        "je 4f",
        // == 4
        "cmp r10, 4",
        "je 5f",
        "2:", // 1 arg
        "   mov rdi, r10",
        "   mov rsi, QWORD PTR [r11]",
        "   jmp 8f",
        "3:", // 2 args
        "   mov rdi, r10",
        "   mov rsi, QWORD PTR [r11]",
        "   mov rdx, QWORD PTR [r11+8]",
        "   jmp 8f",
        "4:", // 3 args
        "   mov rdi, r10",
        "   mov rsi, QWORD PTR [r11]",
        "   mov rdx, QWORD PTR [r11+8]",
        "   mov rcx, QWORD PTR [r11+16]",
        "   jmp 8f",
        "5:", // 4 args
        "   mov rdi, r10",
        "   mov rsi, QWORD PTR [r11]",
        "   mov rdx, QWORD PTR [r11+8]",
        "   mov rcx, QWORD PTR [r11+16]",
        "   mov r8, QWORD PTR [r11+24]",
        "   jmp 8f",
        "6:", // 6+ args needing stack space
        // push last arg, decrement args index (r9)
        "   mov r8, QWORD PTR [r11 + r9]",
        "   push r8",
        "   sub r9, 8", 
        "   cmp r9, 32",
        "   jg 6b",
        "7:", // 5 args 
        // move first args to registers
        "   mov rdi, r10",
        "   mov rsi, QWORD PTR [r11]",
        "   mov rdx, QWORD PTR [r11+8]",
        "   mov rcx, QWORD PTR [r11+16]",
        "   mov r8, QWORD PTR [r11+24]",
        "   mov r9, QWORD PTR [r11+32]",
        "8:",
        // we're not dealing with floating-point, so set eax to 0
        "   xor eax, eax",
        // call variadic fn
        "   call {0}",
        // clean up stack
        "   add rsp, r15",
        sym variadic,
        in("r15") (extra_args + 1) * 8, // keeps track of stack space used for args
        in("r9") (num_args - 1) * 8, // args index
        in("r10") num_args, // number of vargs
        in("r11") args_ptr, // ptr to vargs array
        out("rax") result,
        clobber_abi("C"),
    );

    result
}

unsafe extern "C" fn variadic(n: usize, mut args: ...) -> usize {
    let mut sum = 0;

    for _ in 0..n {
        sum += args.arg::<usize>();
    }

    sum
}
