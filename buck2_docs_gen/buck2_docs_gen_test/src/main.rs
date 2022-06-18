//! This is a test for the doc gen
//! We just verify that this returns some valid looking docs.
//! We can't compile this with the rest of buck2 as the doc registration will be polluted.

use buck2_docs_gen::{Buck2Docs, StarlarkObject};
use starlark::{environment::MethodsBuilder, starlark_module, values::Value};

#[derive(Buck2Docs)]
#[buck2_docs(foo)]
#[allow(unused)]
struct Foo;

#[starlark_module]
fn foo(_builder: &mut MethodsBuilder) {
    /// foo_a docs
    fn foo_a<'v>(this: Value<'v>) -> anyhow::Result<Value<'v>> {
        let _ = this;
        Ok(Value::new_none())
    }
}

#[derive(Buck2Docs)]
#[buck2_docs(bar, name = "bar_name_override")]
#[allow(unused)]
struct Bar;

#[starlark_module]
fn bar(builder: &mut MethodsBuilder) {
    /// bar_b docs
    fn bar_b<'v>(this: Value<'v>) -> anyhow::Result<Value<'v>> {
        let _ = this;
        Ok(Value::new_none())
    }
}

fn main() {
    for (item, doc) in StarlarkObject::all_docs() {
        println!("{}: {:?}", item, doc);
    }
}
