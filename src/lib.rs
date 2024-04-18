
mod errors;
#[allow(unused_imports)]
use extlog::*;
#[allow(unused_imports)]
use extlog::loglib::*;
use std::process::{Command,Stdio,Child,ExitStatus};
//use std::thread::{JoinHandle};
use std::cell::RefCell;
use std::sync::Arc;
use std::error::Error;
use std::io::{Write};

cmdpack_error_class!{CmdPackError}

pub struct CmdExecInner {
	cmds :Vec<String>,
	chld : Option<Child>,
}

impl Drop for CmdExecInner {
	fn drop(&mut self) {
		self.close();
	}
}

impl CmdExecInner {
	fn close(&mut self) {
		if self.chld.is_some() {
			let curchld = self.chld.as_mut().unwrap();
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
			self.chld = None;
		}
		return;
	}

	pub (crate) fn new(cmds :&[String]) -> Result<Self,Box<dyn Error>> {
		let mut retv :Self = Self {
			cmds : Vec::new(),
			chld : None,
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
			chld : None,
		};

		let mut idx :usize = 0;
		while idx < cmds.len() {
			retv.cmds.push(format!("{}",cmds[idx]));
			idx += 1;
		}
		Ok(retv)		
	}

	pub (crate) fn run_bytes(&mut self,inputs :&str) -> Result<(Vec<u8>,Vec<u8>,i32),Box<dyn Error>> {
		if self.cmds.len() == 0 {
			cmdpack_new_error!{CmdPackError,"cmds.len == 0"}
		}
		let mut cmd :Command = Command::new(&self.cmds[0]);
		let mut idx :usize = 1;
		while idx < self.cmds.len() {
			cmd.arg(&self.cmds[idx]);
			idx += 1;
		}


		if inputs.len() == 0{
			cmd.stdin(Stdio::null());
		} else {
			cmd.stdin(Stdio::piped());
		}
		cmd.stdout(Stdio::piped());
		cmd.stderr(Stdio::piped());
		self.chld = Some(cmd.spawn()?);
		if inputs.len() > 0 {
			let  _ = self.chld.as_mut().unwrap().stdin.as_mut().unwrap().write_all(inputs.as_bytes());
			//stdin.write_all(inputs.as_bytes());
		}


		let output = cmd.output()?;
		self.chld = None;
	    let mut exitcode :i32 = -1;
	    let code :Option<i32> = output.status.code();
	    if code.is_some() {
	    	exitcode = code.unwrap();
	    }
	    debug_buffer_trace!(output.stdout.as_ptr(),output.stdout.len(),"{:?} output",self.cmds);
	    debug_buffer_trace!(output.stderr.as_ptr(),output.stderr.len(),"{:?} errout",self.cmds);
	    Ok((output.stdout.clone(),output.stderr.clone(),exitcode))
	}

	pub (crate) fn run(&mut self,inputs :&str) -> Result<(String,String,i32),Box<dyn Error>> {
		let (outb,errb,exitcode) = self.run_bytes(inputs)?;
		let mut outs :String = "".to_string();
		let mut errs :String = "".to_string();
		if outb.len() > 0 {
			let ores = std::str::from_utf8(&outb);
			if ores.is_err() {
				cmdpack_new_error!{CmdPackError,"can not run {:?} output error{:?}",self.cmds,ores.err().unwrap()}
			}
			outs = ores.unwrap().to_string();
		}
		if errb.len() > 0 {
			let ores = std::str::from_utf8(&errb);
			if ores.is_err() {
				cmdpack_new_error!{CmdPackError,"can not run {:?} errout error{:?}",self.cmds,ores.err().unwrap()}
			}
			errs = ores.unwrap().to_string();
		}
		Ok((outs,errs,exitcode))
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

	pub fn run_bytes(&mut self,inputs :&str) -> Result<(Vec<u8>,Vec<u8>,i32),Box<dyn Error>> {
		return self.inner.borrow_mut().run_bytes(inputs);
	}
	pub fn run(&mut self,inputs :&str) -> Result<(String,String,i32),Box<dyn Error>> {
		return self.inner.borrow_mut().run(inputs);
	}

}