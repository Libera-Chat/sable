use super::*;

fn test_handler_zero() -> CommandResult
{
    println!("zero");
    Ok(())
}

fn test_handler_one(x: &u32) -> CommandResult
{
    println!("one");
    assert_eq!(*x, 5);
    Ok(())
}

fn test_handler_two(s: &str) -> CommandResult
{
    println!("two");
    assert_eq!(s, "arg_one");
    Ok(())
}

fn test_handler_three(x: &u32, s1: &str, s2: &str, s3: &str) -> CommandResult
{
    println!("three");
    assert_eq!(*x, 5);
    assert_eq!(s1, "arg_one");
    assert_eq!(s2, "arg_two");
    assert_eq!(s3, "arg_three");
    Ok(())
}

fn test_handler_four(s1: &str, x: &u32, s2: &str, s3: &str) -> CommandResult
{
    println!("four");
    assert_eq!(*x, 5);
    assert_eq!(s1, "arg_one");
    assert_eq!(s2, "arg_two");
    assert_eq!(s3, "arg_three");
    Ok(())
}

fn parsing()
{
    let args = vec![ "arg_one".to_string(), "arg_two".to_string(), "arg_three".to_string() ];

    let ctx = TestCommandContext(5);

    call_handler(&ctx, &test_handler_zero, args.iter().map(AsRef::as_ref)).unwrap();
    call_handler(&ctx, &test_handler_one, args.iter().map(AsRef::as_ref)).unwrap();
    call_handler(&ctx, &test_handler_two, args.iter().map(AsRef::as_ref)).unwrap();
    call_handler(&ctx, &test_handler_three, args.iter().map(AsRef::as_ref)).unwrap();
    call_handler(&ctx, &test_handler_four, args.iter().map(AsRef::as_ref)).unwrap();
}

#[test]
fn parse() { parsing() }