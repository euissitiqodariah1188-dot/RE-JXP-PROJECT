// function.rs - JPX Function Execution
// Rewritten from Python to Rust

use crate::interpreter::value::{JpxFunction, JpxValue};
use crate::interpreter::exceptions::JpxError;
use crate::interpreter::interpreter::JpxInterpreter;

impl JpxInterpreter {
    /// Call a JPX user-defined function with given arguments
    pub fn call_function(&mut self, func: &JpxFunction, args: Vec<JpxValue>) -> Result<JpxValue, JpxError> {
        // Save current environment
        let saved_env = self.env.clone();
        let saved_return = self.return_value.take();

        // Set up new environment from closure
        self.env = func.closure_env.clone();

        // Bind parameters
        for (i, param) in func.params.iter().enumerate() {
            let val = args.get(i).cloned().unwrap_or(JpxValue::Null);
            self.env.insert(param.clone(), val);
        }

        // Reset return value
        self.return_value = None;

        // Execute function body
        let result = self.run_internal(&func.body.clone(), true);

        // Extract return value before restoring env
        let ret_val = self.return_value.take();

        // Restore environment
        self.env = saved_env;
        self.return_value = saved_return;

        match result {
            Ok(_) => Ok(ret_val.unwrap_or(JpxValue::Null)),
            // __return__ signals a return statement was hit — treat as normal exit
            Err(JpxError::Runtime(ref msg)) if msg.starts_with("__return__") => {
                Ok(ret_val.unwrap_or(JpxValue::Null))
            }
            Err(e) => Err(e),
        }
    }
}
