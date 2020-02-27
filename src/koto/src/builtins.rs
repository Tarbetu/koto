use crate::{value, Runtime, Value};
use koto_parser::vec4;
use std::rc::Rc;

pub fn register(runtime: &mut Runtime) {
    use Value::*;

    let builtins = runtime.builtins_mut();

    macro_rules! single_arg_fn {
        ($map_name: ident, $fn_name: expr, $type: ident, $match_name: ident, $body: block) => {
            $map_name.add_fn($fn_name, |args| {
                if args.len() == 1 {
                    match args.first().unwrap() {
                        $type($match_name) => $body
                        unexpected => {
                            return Err(format!(
                                "{}.{} only accepts a {} as its argument, found {}",
                                stringify!($map_name),
                                $fn_name,
                                stringify!($type),
                                value::type_as_string(unexpected)
                            ))
                        }
                    }
                } else {
                    Err(format!(
                        "{}.{} expects one argument, found {}",
                        stringify!($map_name),
                        $fn_name,
                        args.len()
                    ))
                }
            });
        }
    }

    {
        let math = builtins.add_map("math");

        macro_rules! math_fn_1 {
            ($fn:ident) => {
                math_fn_1!(stringify!($fn), $fn)
            };
            ($name:expr, $fn:ident) => {
                single_arg_fn!(math, $name, Number, n, { Ok(Number(n.$fn())) });
            };
        }

        math_fn_1!(abs);
        math_fn_1!(acos);
        math_fn_1!(asin);
        math_fn_1!(atan);
        math_fn_1!(ceil);
        math_fn_1!(cos);
        math_fn_1!(cosh);
        math_fn_1!("degrees", to_degrees);
        math_fn_1!(exp);
        math_fn_1!(exp2);
        math_fn_1!(floor);
        math_fn_1!(log10);
        math_fn_1!(log2);
        math_fn_1!(ln);
        math_fn_1!("radians", to_radians);
        math_fn_1!(recip);
        math_fn_1!(sin);
        math_fn_1!(sinh);
        math_fn_1!(sqrt);
        math_fn_1!(tan);
        math_fn_1!(tanh);
    }

    {
        let list = builtins.add_map("list");

        list.add_fn("add", |args| {
            let mut arg_iter = args.iter();
            let first_arg_value = match arg_iter.next() {
                Some(arg) => arg,
                None => {
                    return Err("Missing list as first argument for list.add".to_string());
                }
            };

            match first_arg_value {
                List(list) => {
                    let mut list = list.clone();
                    let list_data = Rc::make_mut(&mut list);
                    for value in arg_iter {
                        list_data.push(value.clone())
                    }
                    Ok(List(list))
                }
                unexpected => {
                    return Err(format!(
                        "list.add is only supported for lists, found {}",
                        value::type_as_string(unexpected)
                    ))
                }
            }
        });

        single_arg_fn!(list, "is_sortable", List, l, {
            Ok(Bool(list_is_sortable(&l)))
        });

        single_arg_fn!(list, "sort", List, l, {
            if list_is_sortable(l.as_ref()) {
                let mut result = Vec::clone(l);
                result.sort();
                Ok(List(Rc::new(result)))
            } else {
                Err(format!(
                    "list.sort can only sort lists of numbers or strings",
                ))
            }
        });
    }

    {
        let map = builtins.add_map("map");

        single_arg_fn!(map, "keys", Map, m, {
            Ok(List(Rc::new(
                m.as_ref()
                    .keys()
                    .map(|k| Str(k.clone()))
                    .collect::<Vec<_>>(),
            )))
        });
    }

    builtins.add_fn("assert", |args| {
        for value in args.iter() {
            match value {
                Bool(b) => {
                    if !b {
                        return Err("Assertion failed".to_string());
                    }
                }
                _ => return Err("assert only expects booleans as arguments".to_string()),
            }
        }
        Ok(Empty)
    });

    builtins.add_fn("assert_eq", |args| {
        if args.len() != 2 {
            Err(format!(
                "assert_eq expects two arguments, found {}",
                args.len()
            ))
        } else if args[0] == args[1] {
            Ok(Empty)
        } else {
            Err(format!(
                "Assertion failed, '{}' is not equal to '{}'",
                args[0], args[1]
            ))
        }
    });

    builtins.add_fn("assert_ne", |args| {
        if args.len() != 2 {
            Err(format!(
                "assert_ne expects two arguments, found {}",
                args.len()
            ))
        } else if args[0] != args[1] {
            Ok(Empty)
        } else {
            Err(format!(
                "Assertion failed, '{}' should not be equal to '{}'",
                args[0], args[1]
            ))
        }
    });

    builtins.add_fn("length", |args| {
        let mut arg_iter = args.iter();
        let first_arg_value = match arg_iter.next() {
            Some(arg) => arg,
            None => {
                return Err("Missing list as first argument for length".to_string());
            }
        };

        match first_arg_value {
            List(list) => Ok(Number(list.len() as f64)),
            Range { min, max } => Ok(Number((max - min) as f64)),
            unexpected => Err(format!(
                "length is only supported for lists and ranges, found {}",
                unexpected
            )),
        }
    });

    builtins.add_fn("print", |args| {
        for value in args.iter() {
            print!("{}", value);
        }
        println!();
        Ok(Empty)
    });

    builtins.add_fn("vec4", |args| {
        let result = match args {
            [] => vec4::Vec4(0.0, 0.0, 0.0, 0.0),
            [arg] => match arg {
                Number(n) => {
                    let n = *n as f32;
                    vec4::Vec4(n, n, n, n)
                }
                Vec4(v) => *v,
                List(list) => {
                    let mut v = vec4::Vec4::default();
                    for (i, value) in list.iter().take(4).enumerate() {
                        match value {
                            Number(n) => v[i] = *n as f32,
                            unexpected => {
                                return Err(format!(
                                    "vec4 only accepts Numbers as arguments, - found {}",
                                    unexpected
                                ))
                            }
                        }
                    }
                    v
                }
                unexpected => {
                    return Err(format!(
                        "vec4 only accepts a Number, Vec4, or List as first argument - found {}",
                        unexpected
                    ))
                }
            },
            _ => {
                let mut v = vec4::Vec4::default();
                for (i, arg) in args.iter().take(4).enumerate() {
                    match arg {
                        Number(n) => v[i] = *n as f32,
                        unexpected => {
                            return Err(format!(
                                "vec4 only accepts Numbers as arguments, \
                                     or Vec4 or List as first argument - found {}",
                                unexpected
                            ));
                        }
                    }
                }
                v
            }
        };

        Ok(Vec4(result))
    });
}

fn list_is_sortable(list: &Vec<Value>) -> bool {
    use Value::*;

    if list.len() == 0 {
        true
    } else {
        match list.first().unwrap() {
            value @ Number(_) | value @ Str(_) => {
                let value_type = std::mem::discriminant(value);
                list.iter().all(|x| std::mem::discriminant(x) == value_type)
            }
            _ => false,
        }
    }
}
