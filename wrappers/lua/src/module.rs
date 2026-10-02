use alloc::format;
use alloc::string::String;

use crate::Error;
use crate::FromLua;
use crate::Lua;

pub type ModuleLoader = dyn Fn(&str) -> Result<String, Error>;

impl Lua {
    pub fn set_module_loader<F>(&self, loader: F) -> Result<(), Error>
    where
        F: Fn(&str) -> Result<String, Error> + 'static,
    {
        let searcher = self.function_raw(move |lua, args| {
            let nil = lua.nil();
            let name = String::from_lua(args.first().unwrap_or(&nil))?;

            match loader(&name) {
                Ok(src) => lua.load(&src, &name),
                Err(e) => lua.string(&format!("no module '{name}' from the karna loader: {e}")),
            }
        })?;

        self.load("table.insert(package.searchers, 2, ...)", "karna")?
            .call(&[searcher])?;

        Ok(())
    }
}
