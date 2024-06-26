
use std::process::{Command,Child};

#[allow(unused_imports)]
use extargsparse_worker::{extargs_error_class,extargs_new_error};
use std::error::Error;

extargs_error_class!{ProcCmdError}

pub struct RunningCmd {
	cmds :Vec<String>,
	prog :Vec<Command>,
	chld :Vec<Child>,
	exitcode : i32,
}

impl Drop for RunningCmd {
	fn drop(&mut self) {
		self.kill_proc();
	}
}

impl RunningCmd {
	pub fn new(cmds :&[String]) -> Result<Self,Box<dyn Error>> {
		let mut retv :Self = Self {
			cmds : cmds.clone().to_vec(),
			prog : Vec::new(),
			chld : Vec::new(),
			exitcode : -1,
		};

		if cmds.len() == 0 {
			extargs_new_error!{ProcCmdError,"need at least one arg"}
		}
		let mut cmd = Command::new(&retv.cmds[0]);
		let mut idx :usize = 1;
		while idx < retv.cmds.len() {
			cmd.arg(&retv.cmds[idx]);
			idx += 1;
		}
		retv.prog.push(cmd);
		Ok(retv)
	}

	pub fn start(&mut self) -> Result<(),Box<dyn Error>> {
		if self.prog.len() == 0 {
			extargs_new_error!{ProcCmdError,"no prog set"}
		}
		if self.chld.len() == 0   {
			let ores = self.prog[0].spawn();
			if ores.is_err() {
				extargs_new_error!{ProcCmdError,"spawn {:?} error {:?}",self.cmds,ores.err().unwrap()}
			}
			let chld = ores.unwrap();
			self.chld.push(chld);
		}
		Ok(())
	}

	pub fn is_running(&mut self) -> bool {
		if self.chld.len() == 0 {
			return false;
		}

		let ores = self.chld[0].try_wait();
		if ores.is_err() {
			return true;
		}
		let bval = ores.unwrap();
		if bval.is_none() {
			return true;
		}
		let exitsts = bval.unwrap();
		let code = exitsts.code();
		if code.is_none() {
			/*code is none*/
			let _ = self.chld.pop();
			self.exitcode = -1;
			return false;
		}
		

		self.exitcode = code.unwrap();
		/*to remove the value*/
		let _ = self.chld.pop();
		return false;
	}

	pub fn kill_proc(&mut self) {
		while self.is_running() {
			let _ = self.chld[0].kill();
			std::thread::sleep(std::time::Duration::from_millis(10));
		}
		return;
	}

	pub fn exitcode(&self) -> i32 {
		return self.exitcode;
	}
}