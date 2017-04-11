use parser::Identifier;
use memory::MemoryBlock;

use operations::{Error, Operation, OperationsResult};
use operations::scope::{TypeId, ScopeStack, ScopeItem};

use super::number::store_number;

pub fn store_identifier(
    scope: &mut ScopeStack,
    name: Identifier,
    target_type: TypeId,
    target: MemoryBlock,
) -> OperationsResult {
    scope.lookup(&name).first().ok_or_else(|| {
        Error::UnresolvedName(name.clone())
    }).map(|item| (**item).clone()).and_then(|item| match item {
        // There is a non-lexical lifetimes issue here which was introduced by calling store_number() below
        // The clone() above is completely unnecssary and is a hack to work around this problem
        // in the Rust compiler
        // http://smallcultfollowing.com/babysteps/blog/2016/04/27/non-lexical-lifetimes-introduction/#problem-case-2-conditional-control-flow
        ScopeItem::Constant {type_id, ref bytes} => {
            if target_type == type_id {
                increment_to_value(target, bytes)
            }
            else {
                mismatched_types(scope, target_type, type_id)
            }
        },

        ScopeItem::NumericLiteral(number) => store_number(scope, number, target_type, target),

        ScopeItem::TypedBlock {type_id, memory} => {
            if target_type == type_id {
                // Need to check this invariant or else this can lead to
                // many very subtle bugs
                debug_assert!(memory.size() == target.size());

                Ok(vec![Operation::Copy {
                    source: memory.position(),
                    target: target.position(),
                    size: memory.size(),
                }])
            }
            else {
                mismatched_types(scope, target_type, type_id)
            }
        },

        ScopeItem::Array {item, size, memory} => {
            unimplemented!();
        },

        ScopeItem::BuiltInFunction { .. } => {
            // This is not supported for now
            unreachable!();
        },
    })
}

fn increment_to_value(mem: MemoryBlock, value: &Vec<u8>) -> OperationsResult {
    debug_assert!(mem.size() == value.len());

    Ok(value.iter().enumerate().map(|(i, &byte)| {
        Operation::Increment {
            target: mem.position_at(i),
            amount: byte as usize,
        }
    }).collect())
}

fn mismatched_types(scope: &ScopeStack, expected: TypeId, found: TypeId) -> OperationsResult {
    Err(Error::MismatchedTypes {
        expected: scope.get_type(expected).clone(),
        found: scope.get_type(found).clone(),
    })
}