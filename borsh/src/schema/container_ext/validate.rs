use super::{is_zero_size, ZeroSizeError};
use super::{BorshSchemaContainer, Declaration, Definition, Fields};
use crate::__private::maybestd::{string::ToString, vec::Vec};

impl BorshSchemaContainer {
    /// Validates container for violation of any well-known rules with
    /// respect to `borsh` serialization.
    ///
    /// # Example
    ///
    /// ```
    /// use borsh::schema::BorshSchemaContainer;
    ///
    /// let schema = BorshSchemaContainer::for_type::<usize>();
    /// assert_eq!(Ok(()), schema.validate());
    /// ```
    pub fn validate(&self) -> core::result::Result<(), Error> {
        let mut stack = Vec::new();
        validate_impl(self.declaration(), self, &mut stack)
    }
}

/// Possible error when validating a [`BorshSchemaContainer`], generated for some type `T`,
/// for violation of any well-known rules with respect to `borsh` serialization.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Error {
    /// sequences of zero-sized types of dynamic length are forbidden by definition
    /// see <https://github.com/near/borsh-rs/pull/202> and related ones
    ZSTSequence(Declaration),
    /// Declared tag width is too large.  Tags may be at most eight bytes.
    TagTooWide(Declaration),
    /// Declared tag width is too small.  Tags must be large enough to represent
    /// possible length of sequence.
    TagTooNarrow(Declaration),
    /// only 0, 1, 2, 4 and 8 bytes long enum tags and sequences' `length_width` are alowed
    TagNotPowerOfTwo(Declaration),
    /// Some of the declared types were lacking definition, which is considered
    /// a container's validation error
    MissingDefinition(Declaration),
    /// A Sequence defined with an empty length range.
    EmptyLengthRange(Declaration),
}

fn check_length_width(declaration: &Declaration, width: u8, max: u64) -> Result<(), Error> {
    match width {
        0 => Ok(()),
        3 | 5 | 6 | 7 => Err(Error::TagNotPowerOfTwo(declaration.clone())),
        1..=7 if max < 1 << (width * 8) => Ok(()),
        1..=7 => Err(Error::TagTooNarrow(declaration.clone())),
        8 => Ok(()),
        _ => Err(Error::TagTooWide(declaration.clone())),
    }
}

const U64_LEN: u8 = 8;

fn validate_impl<'a>(
    declaration: &'a Declaration,
    schema: &'a BorshSchemaContainer,
    stack: &mut Vec<&'a Declaration>,
) -> core::result::Result<(), Error> {
    let definition = match schema.get_definition(declaration) {
        Some(definition) => definition,
        None => {
            return Err(Error::MissingDefinition(declaration.to_string()));
        }
    };
    if stack.iter().any(|dec| *dec == declaration) {
        return Ok(());
    }
    stack.push(declaration);
    match definition {
        Definition::Primitive(_size) => {}
        // arrays branch
        Definition::Sequence {
            length_width,
            length_range,
            elements,
        } if *length_width == Definition::ARRAY_LENGTH_WIDTH
            && length_range.clone().count() == 1 =>
        {
            validate_impl(elements, schema, stack)?
        }
        Definition::Sequence {
            length_width,
            length_range,
            elements,
        } => {
            if length_range.is_empty() {
                return Err(Error::EmptyLengthRange(declaration.clone()));
            }
            check_length_width(declaration, *length_width, *length_range.end())?;
            match is_zero_size(elements, schema) {
                Ok(true) => return Err(Error::ZSTSequence(declaration.clone())),
                Ok(false) => (),
                // a recursive type either has no exit, so it cannot be instantiated
                // or it uses `Definiotion::Enum` or `Definition::Sequence` to exit from recursion
                // which make it non-zero size
                Err(ZeroSizeError::Recursive) => (),
                Err(ZeroSizeError::MissingDefinition(declaration)) => {
                    return Err(Error::MissingDefinition(declaration));
                }
            }
            validate_impl(elements, schema, stack)?;
        }
        Definition::Enum {
            tag_width,
            variants,
        } => {
            if *tag_width > U64_LEN {
                return Err(Error::TagTooWide(declaration.to_string()));
            }
            for (_, variant) in variants {
                validate_impl(variant, schema, stack)?;
            }
        }
        Definition::Tuple { elements } => {
            for element_type in elements {
                validate_impl(element_type, schema, stack)?;
            }
        }
        Definition::Struct { fields } => match fields {
            Fields::NamedFields(fields) => {
                for (_field_name, field_type) in fields {
                    validate_impl(field_type, schema, stack)?;
                }
            }
            Fields::UnnamedFields(fields) => {
                for field_type in fields {
                    validate_impl(field_type, schema, stack)?;
                }
            }
            Fields::Empty => {}
        },
    };
    stack.pop();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{check_length_width, Error};
    use crate::__private::maybestd::string::ToString;

    #[test]
    fn test_check_tag_width() {
        assert_eq!(check_length_width(&"test".to_string(), 0, u64::MAX), Ok(()));
        assert_eq!(
            check_length_width(&"test".to_string(), 1, u8::MAX as u64),
            Ok(())
        );
        assert_eq!(
            check_length_width(&"test".to_string(), 1, u8::MAX as u64 + 1),
            Err(Error::TagTooNarrow("test".to_string()))
        );

        assert_eq!(
            check_length_width(&"test".to_string(), 2, u16::MAX as u64),
            Ok(())
        );
        assert_eq!(
            check_length_width(&"test".to_string(), 2, u16::MAX as u64 + 1),
            Err(Error::TagTooNarrow("test".to_string()))
        );

        assert_eq!(
            check_length_width(&"test".to_string(), 3, 100),
            Err(Error::TagNotPowerOfTwo("test".to_string()))
        );

        assert_eq!(
            check_length_width(&"test".to_string(), 4, u32::MAX as u64),
            Ok(())
        );

        assert_eq!(
            check_length_width(&"test".to_string(), 4, u32::MAX as u64 + 1),
            Err(Error::TagTooNarrow("test".to_string()))
        );
        assert_eq!(
            check_length_width(&"test".to_string(), 5, 100),
            Err(Error::TagNotPowerOfTwo("test".to_string()))
        );

        assert_eq!(
            check_length_width(&"test".to_string(), 6, 100),
            Err(Error::TagNotPowerOfTwo("test".to_string()))
        );

        assert_eq!(
            check_length_width(&"test".to_string(), 7, 100),
            Err(Error::TagNotPowerOfTwo("test".to_string()))
        );

        assert_eq!(check_length_width(&"test".to_string(), 8, u64::MAX), Ok(()));
    }
}
