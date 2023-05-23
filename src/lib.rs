//! # Smali
//!
//! A library for reading and writing Android smali files
//!
use std::collections::HashSet;
use std::path::PathBuf;
use nom::{IResult, multi::{many_till, many0}, sequence::terminated, combinator::eof};
use smali_parse::{blank_line, parse_instruction};
use types::SmaliInstruction;
use crate::types::{SmaliClass, SmaliError};

pub mod types;
mod smali_parse;
mod smali_write;

/// Recurses a base path, typically a 'smali' folder from apktool returning a Vector of all found smali classes
///
/// # Examples
///
/// ```
///  use smali::find_smali_files;
///
/// let mut p = PathBuf::from_str("smali")?;
///  let mut classes = find_smali_files(&p)?;
///  println!("{:} smali classes loaded.", classes.len());
/// ```
pub fn find_smali_files(dir: &PathBuf) -> Result<Vec<SmaliClass>, SmaliError>
{
    let mut results = vec![];

    for p in dir.read_dir().unwrap()
    {
        if let Ok(p) = p
        {
            // Directory: recurse sub-directory
            if let Ok(f) = p.file_type()
            {
                if f.is_dir() {
                    let mut new_dir = dir.clone();
                    new_dir.push(p.file_name());
                    let dir_hs = find_smali_files(&new_dir)?;
                    results.extend(dir_hs);
                } else {
                    // It's a smali file
                    if p.file_name().to_str().unwrap().ends_with(".smali")
                    {
                        let dex_file = SmaliClass::read_from_file(&p.path())?;
                        results.push(dex_file);
                    }
                }
            }
        }
    }

    Ok(results)
}

pub fn parse_fragment(input: &str) -> Result<Vec<SmaliInstruction>, SmaliError>
{
    match many_till(terminated(parse_instruction, many0(blank_line)), eof)(input) {
        IResult::Ok((_, (instructions, _))) => Ok(instructions),
        IResult::Err(nom::Err::Failure(e)) => Err(SmaliError { details: format!("parse error at {:?}", e.to_string()) }),
        _ => Err(SmaliError { details: "unknown parse error".to_string() })
    }
}


#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use crate::types::{MethodSignature, ObjectIdentifier, SmaliClass, TypeSignature};

    #[test]
    fn object_identifier_to_jni() {
        let o = ObjectIdentifier::from_java_type("com.basic.Test");
        assert_eq!(o.as_java_type(), "com.basic.Test");
        assert_eq!(o.as_jni_type(), "Lcom/basic/Test;");
    }

    #[test]
    fn object_identifier_to_java() {
        let o = ObjectIdentifier::from_jni_type("Lcom/basic/Test;");
        assert_eq!(o.as_jni_type(), "Lcom/basic/Test;");
        assert_eq!(o.as_java_type(), "com.basic.Test");
    }

    #[test]
    fn signatures() {
        let t = TypeSignature::Bool;
        assert_eq!(t.to_jni(), "Z");
        let m = MethodSignature::from_jni("([I)V");
        assert_eq!(m.return_type, TypeSignature::Void);
    }

    #[test]
    fn parse_write() {
        let dex = SmaliClass::read_from_file(Path::new("tests/OkHttpClient.smali")).unwrap();
        let smali = dex.to_smali();

        // Attempt to parse the output
        let dex = SmaliClass::from_smali(&smali).unwrap();
        println!("{}\n", dex.to_smali());
    }
}
