use parser::{Pattern, TypeDefinition, Expression};

use super::{Operation, OperationsResult, type_definition, expression};
use super::scope::ScopeStack;

pub fn into_operations(
    scope: &mut ScopeStack,
    pattern: Pattern,
    type_def: TypeDefinition,
    expr: Option<Expression>,
) -> OperationsResult {
    let type_id = type_definition::resolve_type_id(scope, type_def)?;

    let name = match pattern {
        Pattern::Identifier(name) => name,
    };

    // Need to always declare the variable in the scope
    let mem = scope.declare(name, type_id);
    let mut ops = vec![Operation::Allocate(mem)];

    if let Some(expr) = expr {
        ops.extend(expression::into_operations(scope, expr, type_id, mem)?)
    }

    Ok(ops)
}

#[cfg(test)]
mod tests {
    use super::*;

    use parser::Identifier;
    use operations::item_type::ItemType;

    #[test]
    fn declaration_only() {
        // When only doing a declaration, no operations should be generated
        // since there is no expression to actually evaluate
        let mut scope = ScopeStack::new();
        scope.declare_type(Identifier::from("u8"), ItemType::Primitive(1));

        let ops = into_operations(
            &mut scope,
            Pattern::Identifier(Identifier::from("foo")),
            TypeDefinition::Name {name: Identifier::from("u8")},
            None
        ).unwrap();

        assert!(!scope.lookup(&Identifier::from("foo")).is_empty(), "No value was declared");
        assert_eq!(ops.len(), 1);
    }
}
