// yet another brainfuck interpreter, blah-blah-blah

fn main() {
	unsafe {
		bf::interpret("++++++++++[>+++++++>++++++++++>+++>+<<<<-]>++.>+.+++++++..+++.>++.<<+++++++++++++++.>.+++.------.--------.>+.>.");
	}
}

mod bf {
	use std::alloc::{self, Layout};
	use std::io::{self, Read};
	use std::slice;
	use libc::getchar;

	pub unsafe fn interpret(program: &str) {
		let relevant_layout = Layout::new::<char>();
		let mut original_ptr = alloc::alloc(relevant_layout);
		let mut ptr = original_ptr;
		let mut ip: usize = 0;
		let mut last_jmp_ip: usize = usize::MAX;
		let chs: Vec<char> = program.chars().collect();
		loop {
			if ip == chs.len() {
				break;
			}
			// fetch instruction
			let c = chs[ip];
			match c {
				'>' => { ptr = ptr.offset(1); ip += 1 },
				'<' => { ptr = ptr.offset(-1); ip += 1} ,
				'+' => {*ptr += 1; ip += 1 },
				'-' => { *ptr -= 1; ip += 1},
				'.' => { print!("{}", *ptr as char); ip += 1},
				',' => { *ptr = getchar() as u8; ip += 1 },
				'[' => {
					last_jmp_ip = ip;
					ip += 1
				},
				']' => {
					if *ptr > 0 && last_jmp_ip != usize::MAX {
						ip = last_jmp_ip;
						ip += 1;
						continue;
					}
					last_jmp_ip = usize::MAX;
					ip += 1;
				}
				_ => panic!("unknown token {}!!!", c),
			}
		}

		alloc::dealloc(original_ptr, relevant_layout);
	}

	pub unsafe fn execute_while_block(program: &str, ptr: &mut *mut u8) {

	} 
}
