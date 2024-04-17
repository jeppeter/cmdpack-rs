
mod errors;
#[allow(unused_imports)]
use extlog::*;
#[allow(unused_imports)]
use extlog::loglib::*;
use std::process::{Command,Stdio};
use std::cell::RefCell;
use std::sync::Arc;
use std::error::Error;

cmdpack_error_class!{CmdPackError}

pub struct CmdExecInner {
	cmds :Vec<String>,
}

impl CmdExecInner {
	pub (crate) fn new(cmds :&[String]) -> Result<Self,Box<dyn Error>> {
		let mut retv :Self = Self {
			cmds : Vec::new(),
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
		};

		let mut idx :usize = 0;
		while idx < cmds.len() {
			retv.cmds.push(format!("{}",cmds[idx]));
			idx += 1;
		}
		Ok(retv)		
	}

	pub (crate) fn run_bytes(&self,inputs :&str) -> Result<(Vec<u8>,Vec<u8>,i32),Box<dyn Error>> {
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
		}
		let output = cmd.output()?;
	    let mut exitcode :i32 = -1;
	    let code :Option<i32> = output.status.code();
	    if code.is_some() {
	    	exitcode = code.unwrap();
	    }
	    debug_buffer_trace!(output.stdout.as_ptr(),output.stdout.len(),"{:?} output",self.cmds);
	    debug_buffer_trace!(output.stderr.as_ptr(),output.stderr.len(),"{:?} errout",self.cmds);
	    Ok((output.stdout.clone(),output.stderr.clone(),exitcode))
	}

	pub (crate) fn run(&self,inputs :&str) -> Result<(String,String,i32),Box<dyn Error>> {
		let (outb,errb,exitcode) = self.run_bytes(inputs)?;
		let outs = std::str::from_utf8(&outb)?;
		let errs = std::str::from_utf8(&errb)?;
		Ok((outs.to_string(),errs.to_string(),exitcode))
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

	pub fn run_bytes(&self,inputs :&str) -> Result<(Vec<u8>,Vec<u8>,i32),Box<dyn Error>> {
		return self.inner.borrow().run_bytes(inputs);
	}
	pub fn run(&self,inputs :&str) -> Result<(String,String,i32),Box<dyn Error>> {
		return self.inner.borrow().run(inputs);
	}

}