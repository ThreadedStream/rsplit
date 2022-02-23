
fn main() {
	unsafe {
		bf::interpret("<<<<>>>>");
	}
}

mod bf {
	use std::alloc::{self, Layout};
	use std::io::{self, Read};
	use std::slice;

	pub unsafe fn interpret(program: &str) {
		let relevant_layout = Layout::new::<u8>();
		let mut ptr = alloc::alloc(relevant_layout);
		let mut ip: usize = 0;
		let mut last_jmp_ip: usize = 0;
		loop {
			// fetch instruction
			let c = chs[ip];
			match c {
				'>' => ptr = ptr.offset(1),
				'<' => ptr = ptr.offset(-1),
				'+' => *ptr += 1,
				'-' => *ptr -= 1,
				'.' => print!("{}", *ptr),
				',' => *ptr = std::io::stdin().bytes().next().and_then(|result| result.ok()).unwrap(),
				'[' => {
					last_jmp_ip = ip;
					ip+=1;
				},
				_ => panic!("unknown token {}!!!", c),
			}
		}
		// for c in program.chars() {
		// 	match c {
		// 		'>' => ptr = ptr.offset(1),
		// 		'<' => ptr = ptr.offset(-1),
		// 		'+' => *ptr += 1,
		// 		'-' => *ptr -= 1,
		// 		'.' => print!("{}", *ptr),
		// 		',' => *ptr = std::io::stdin().bytes().next().and_then(|result| result.ok()).unwrap(),
		// 		'[' => {
		// 			loop {
						
		// 			}
		// 		}
		// 		_ => panic!("unknown token {}!!!", c),
		// 	}
		// }

		alloc::dealloc(ptr, relevant_layout);
	}

	pub unsafe fn execute_while_block(program: &str, ptr: &mut *mut u8) {

	} 
}
