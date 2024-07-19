use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::FloatPredicate;
use std::env;
use std::error::Error;

type Fun = unsafe extern "C" fn(f64, f64) -> bool;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let inputs: Vec<String> = args[1..].to_vec();

    let mut float_args: Vec<f64> = Vec::new();
    for arg in inputs {
        match arg.parse::<f64>() {
            Ok(value) => float_args.push(value),
            Err(e) => {
                eprintln!("Error parsing '{}': {}", arg, e);
                return Err(Box::new(e))
            }
        }
    }

    let lhs = match float_args.get(0) {
        Some(v) => v,
        None => &9.11,
    };
    let rhs = match float_args.get(1) {
        Some(v) => v,
        None => &9.9,
    };

    let context = Context::create();
    let module = context.create_module("nine_eleven");
    let builder = context.create_builder();

    if let Err(err) = generate_comparison(&context, &module, &builder) {
        eprintln!("generate comparison error: {}", err.to_string());
        return Err(err);
    }

    let execution_engine = module.create_jit_execution_engine(inkwell::OptimizationLevel::None)?;

    let fun: inkwell::execution_engine::JitFunction<Fun> =
        unsafe { execution_engine.get_function("main") }?;
    let result = unsafe { fun.call(*lhs, *rhs) };

    println!("Is {} larger than {} ? {}", lhs, rhs, result);

    Ok(())
}

fn generate_comparison<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
) -> Result<(), Box<dyn Error>> {
    let f64_type = context.f64_type();
    let i32_type = context.i32_type();
    // let i8_type = context.i8_type();
    let bool_type = context.bool_type();

    let ptr_type = context.ptr_type(0.into());

    let fn_type = bool_type.fn_type(&[f64_type.into(), f64_type.into()], false);
    let function = module.add_function("main", fn_type, None);

    let printf_type = i32_type.fn_type(&[ptr_type.into()], true);
    let printf = module.add_function("printf", printf_type, None);

    let basic_block = context.append_basic_block(function, "entry");
    builder.position_at_end(basic_block);

    let param1 = function.get_nth_param(0).unwrap().into_float_value();
    let param2 = function.get_nth_param(1).unwrap().into_float_value();

    let comparison = builder.build_float_compare(FloatPredicate::OGT, param1, param2, "compare")?;

    let cmp_i32 = builder.build_int_z_extend(comparison, i32_type, "compare_i32")?;

    let format_str = builder.build_global_string_ptr("%d\n", "format_str")?;

    builder.build_call(
        printf,
        &[format_str.as_pointer_value().into(), cmp_i32.into()],
        "printf_call",
    )?;

    builder.build_return(Some(&comparison))?;

    module.print_to_file("output.ll")?;

    if let Err(err) = module.verify() {
        return Err(Box::new(err));
    }

    Ok(())
}
