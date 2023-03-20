use inkwell::context::Context;
use inkwell::types::{BasicType, BasicTypeEnum};

pub fn get_internal_struct<'ctx>(context: &'ctx Context, name: &String) -> (u32, BasicTypeEnum<'ctx>) {
    return match name.as_str() {
        "i64" => (8, context.i64_type().as_basic_type_enum()),
        "i32" => (4, context.i32_type().as_basic_type_enum()),
        "i16" => (2, context.i16_type().as_basic_type_enum()),
        "i8" => (1, context.i8_type().as_basic_type_enum()),
        "bool" => (1, context.bool_type().as_basic_type_enum()),
        _ => panic!("Unknown internal type: {}", name)
    };
}