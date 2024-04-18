#[allow(unused_imports)]
use extargsparse_codegen::{extargs_load_commandline,ArgSet,extargs_map_function};
#[allow(unused_imports)]
use extargsparse_worker::{extargs_error_class,extargs_new_error};
#[allow(unused_imports)]
use extargsparse_worker::namespace::{NameSpaceEx};
#[allow(unused_imports)]
use extargsparse_worker::options::{ExtArgsOptions};
#[allow(unused_imports)]
use extargsparse_worker::argset::{ArgSetImpl};
use extargsparse_worker::parser::{ExtArgsParser};
use extargsparse_worker::funccall::{ExtArgsParseFunc};
#[allow(unused_imports)]
use extargsparse_worker::const_value::{COMMAND_SET,SUB_COMMAND_JSON_SET,COMMAND_JSON_SET,ENVIRONMENT_SET,ENV_SUB_COMMAND_JSON_SET,ENV_COMMAND_JSON_SET,DEFAULT_SET};


#[allow(unused_imports)]
use std::cell::RefCell;
#[allow(unused_imports)]
use std::sync::Arc;
#[allow(unused_imports)]
use std::error::Error;
use std::boxed::Box;
#[allow(unused_imports)]
use regex::Regex;
#[allow(unused_imports)]
use std::any::Any;
use lazy_static::lazy_static;
use std::collections::HashMap;

use cmdpack::*;
use extlog::*;
use extlog::loglib::*;

mod logtrans;
#[allow(dead_code)]
mod strop;
mod fileop;

extargs_error_class!{CmdPackError}


fn run_handler(ns :NameSpaceEx,_optargset :Option<Arc<RefCell<dyn ArgSetImpl>>>,_ctx :Option<Arc<RefCell<dyn Any>>>) -> Result<(),Box<dyn Error>> {	
	let sarr :Vec<String> ;
	let mut inputs :String = "".to_string();
	let infile :String;

	logtrans::init_log(ns.clone())?;
	sarr = ns.get_array("subnargs");
	if sarr.len() < 1 {
		extargs_new_error!{CmdPackError,"need args"}
	}
	infile = ns.get_string("input");
	if infile.len() > 0 {
		inputs = fileop::read_file(&infile)?;
	}

	let cmd :CmdExec = CmdExec::new(&sarr)?;
	let (outs,errs,exitcode) = cmd.run(&inputs)?;
	debug_trace!("run {:?} exitcode[{}]",sarr,exitcode);
	debug_trace!("outs\n{}",outs);
	debug_trace!("errs\n{}",errs);

	Ok(())
}


#[extargs_map_function(run_handler)]
fn main() -> Result<(),Box<dyn Error>> {
	let parser :ExtArgsParser = ExtArgsParser::new(None,None)?;
	let commandline = r#"
	{
		"output|o" : null,
		"input|i" : null,
		"run<run_handler>##args ... to run command##" : {
			"$" : "+"
		}
	}
	"#;
	extargs_load_commandline!(parser,commandline)?;
	logtrans::prepare_log(parser.clone())?;
	let ores = parser.parse_commandline_ex(None,None,None,None);
	if ores.is_err() {
		let e = ores.err().unwrap();
		eprintln!("{:?}", e);
		return Err(e);
	}
	return Ok(());
}
