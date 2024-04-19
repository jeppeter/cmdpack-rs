
mod errors;
#[allow(unused_imports)]
use extlog::*;
#[allow(unused_imports)]
use extlog::loglib::*;
use std::process::{Command,Stdio,Child,ExitStatus};
use std::thread::{JoinHandle};
use std::cell::RefCell;
use std::sync::Arc;
use std::error::Error;
use std::io::{Write};

cmdpack_error_class!{CmdPackError}

pub struct CmdExecInner {
	cmds :Vec<String>,
	chld : Vec<Child>,
	thropt :Vec<JoinHandle<()>>,
}

impl Drop for CmdExecInner {
	fn drop(&mut self) {
		self.close();
	}
}

impl CmdExecInner {
	fn close(&mut self) {
		while self.chld.len() > 0 {
			let mut curchld = self.chld.pop().unwrap();
			loop {
				let ores = curchld.try_wait();
				if ores.is_ok() {
					let ow :Option<ExitStatus> = ores.unwrap();
					if ow.is_some() {
						break;
					}
				}

				let _ =curchld.kill();
			}
		}

		while self.thropt.len() > 0 {
			let jp = self.thropt.pop().unwrap();
			let _ = jp.join();
		}
		return;
	}

	pub (crate) fn new(cmds :&[String]) -> Result<Self,Box<dyn Error>> {
		let mut retv :Self = Self {
			cmds : Vec::new(),
			chld : Vec::new(),
			thropt : Vec::new(),
		};

		let mut idx :usize = 0;
		while idx < cmds.len() {
			retv.cmds.push(format!("{}",cmds[idx]));
			idx += 1;
		}
		Ok(retv)
	}

	pub (crate) fn new_str(cmds :&[&str]) -> Result<Self,Box<dyn Error>> {
		let mut retv :Self = Self {
			cmds : Vec::new(),
			chld : Vec::new(),
			thropt : Vec::new(),
		};

		let mut idx :usize = 0;
		while idx < cmds.len() {
			retv.cmds.push(format!("{}",cmds[idx]));
			idx += 1;
		}
		Ok(retv)		
	}

	pub (crate) fn run_bytes(&mut self,inputb :&[u8]) -> Result<(Vec<u8>,Vec<u8>,i32),Box<dyn Error>> {
		if self.cmds.len() == 0 {
			cmdpack_new_error!{CmdPackError,"cmds.len == 0"}
		}
		if self.chld.len() > 0 {
			cmdpack_new_error!{CmdPackError,"{:?} still running",self.cmds}
		}
		let mut cmd :Command = Command::new(&self.cmds[0]);
		let mut idx :usize = 1;
		while idx < self.cmds.len() {
			cmd.arg(&self.cmds[idx]);
			idx += 1;
		}


		if inputb.len() == 0{
			cmd.stdin(Stdio::null());
		} else {
			cmd.stdin(Stdio::piped());
		}
		cmd.stdout(Stdio::piped());
		cmd.stderr(Stdio::piped());
		self.chld.push(cmd.spawn()?);
		if inputb.len() > 0 {
			if inputb.len() > 512 {
				debug_buffer_trace!(inputb.as_ptr(),512,"write out buffer top [512]");
				debug_buffer_trace!(inputb[(inputb.len()-512)..].as_ptr(),512,"write out buffer end [512]");
			} else {
				debug_buffer_trace!(inputb.as_ptr(),inputb.len(),"write out buffer");	
			}
			
			let cb :Vec<u8> = inputb.to_vec();
			let mut stdin = self.chld[0].stdin.take().unwrap();
			self.thropt.push(std::thread::spawn(move || {
				let mut writed :usize = 0;
				let mut cursize :usize;
				loop {
					if writed >= cb.len() {
						break;
					}
					debug_trace!("writed {}:0x{:x}",writed,writed);

					let ores = stdin.write(&cb[writed..]);
					if ores.is_err() {
						break;
					}
					cursize = ores.unwrap();
					debug_trace!("write {}:0x{:x} size",cursize,cursize);
					writed += cursize;
				}
				drop(stdin);
			}));

			//stdin.write_all(inputs.as_bytes());
		}


		let curchld = self.chld.pop().unwrap();
		let output = curchld.wait_with_output()?;
		let thr = self.thropt.pop().unwrap();
		let _ = thr.join();
	    let mut exitcode :i32 = -1;
	    let code :Option<i32> = output.status.code();
	    if code.is_some() {
	    	exitcode = code.unwrap();
	    }
	    //debug_buffer_trace!(output.stdout.as_ptr(),output.stdout.len(),"{:?} output",self.cmds);
	    //debug_buffer_trace!(output.stderr.as_ptr(),output.stderr.len(),"{:?} errout",self.cmds);
	    Ok((output.stdout.clone(),output.stderr.clone(),exitcode))
	}


}


#[derive(Clone)]
pub struct CmdExec {
	inner :Arc<RefCell<CmdExecInner>>,
}

impl Drop for CmdExec {
	fn drop(&mut self) {
		self.close();
	}
}

impl CmdExec {
	pub fn close(&mut self) {
		debug_trace!("CmdExec close");
	}
	pub fn new(cmds :&[String]) -> Result<Self,Box<dyn Error>> {
		let retv :Self = Self {
			inner : Arc::new(RefCell::new(CmdExecInner::new(cmds)?)),
		};
		Ok(retv)
	}

	pub fn new_str(cmds :&[&str]) -> Result<Self,Box<dyn Error>> {
		let retv :Self = Self {
			inner : Arc::new(RefCell::new(CmdExecInner::new_str(cmds)?)),
		};
		Ok(retv)		
	}

	pub fn run_bytes(&mut self,inputs :&[u8]) -> Result<(Vec<u8>,Vec<u8>,i32),Box<dyn Error>> {
		return self.inner.borrow_mut().run_bytes(inputs);
	}
}