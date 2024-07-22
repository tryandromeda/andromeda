use std::borrow::BorrowMut;

use nova_vm::ecmascript::{
    builtins::ArgumentsList,
    execution::{Agent, JsResult},
    types::Value,
};

/// Debug function that prints the first argument to the console.
pub fn debug(agent: &mut Agent, _this: Value, args: ArgumentsList) -> JsResult<Value> {
    if args.len() == 0 {
        println!();
    } else {
        println!("{}", args[0].to_string(agent)?.as_str(agent));
    }
    Ok(Value::Undefined)
}

/// Read a text file and return the content as a string.
pub fn internal_read_text_file(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
) -> JsResult<Value> {
    let binding = args.get(0).to_string(agent)?;
    let path = binding.as_str(agent);
    let content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => {
            return Ok(Value::from_string(
                agent,
                format!("Error: {}", e.to_string()),
            ));
        }
    };
    Ok(Value::from_string(agent, content))
}

/// Write a text file with the content of the second argument.
pub fn internal_write_text_file(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
) -> JsResult<Value> {
    let binding = args.get(0).to_string(agent)?;
    let content = args.get(1).to_string(agent.borrow_mut())?;
    match std::fs::write(binding.as_str(agent), content.as_str(agent)) {
        Ok(_) => Ok(Value::from_string(agent, "Success".to_string())),
        Err(e) => Ok(Value::from_string(
            agent,
            format!("Error: {}", e.to_string()),
        )),
    }
}

/// Exit the process with the given exit code.
pub fn internal_exit(agent: &mut Agent, _this: Value, args: ArgumentsList) -> JsResult<Value> {
    std::process::exit(args[0].to_int32(agent)?);
}

/// Internal read for reading from the console.
pub fn internal_read(agent: &mut Agent, _this: Value, _args: ArgumentsList) -> JsResult<Value> {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    Ok(Value::from_string(agent, input.trim_end().to_string()))
}

/// Internal read line for reading from the console with a newline.
pub fn internal_read_line(
    agent: &mut Agent,
    _this: Value,
    _args: ArgumentsList,
) -> JsResult<Value> {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    Ok(Value::from_string(agent, input))
}

/// Internal write for writing to the console.
pub fn internal_write(agent: &mut Agent, _this: Value, args: ArgumentsList) -> JsResult<Value> {
    for arg in args.iter() {
        print!("{}", arg.to_string(agent)?.as_str(agent));
    }
    Ok(Value::Undefined)
}

/// Internal write line for writing to the console with a newline.
pub fn internal_write_line(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
) -> JsResult<Value> {
    for arg in args.iter() {
        print!("{}", arg.to_string(agent)?.as_str(agent));
    }
    println!();
    Ok(Value::Undefined)
}
