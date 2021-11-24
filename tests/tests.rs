use parse_fmt_str::{parse_fmt_str, PossibleFormatSlot, Type, Align, Sign, Count, Argument, Percision};

#[test]
fn hello_fmt_str() {
    let formatstr = parse_fmt_str("Hello, {:}!").unwrap();
    assert_eq!(formatstr.text, vec!["Hello, ", "!"]);
    assert_eq!(formatstr.maybe_fmt.len(), 1);
    assert!(match &formatstr.maybe_fmt[0] {
	PossibleFormatSlot::FormatSlot(fmt) => {
	    assert!(fmt.arg.is_none());
	    assert!(fmt.fmt_spec.is_some());
	    let fmt_spec = fmt.fmt_spec.as_ref().unwrap();
	    assert!(fmt_spec.fill.is_none());
	    assert!(fmt_spec.align.is_none());
	    assert!(fmt_spec.sign.is_none());
	    assert!(!fmt_spec.alternate);
	    assert!(!fmt_spec.pad_with_zeros);
	    assert!(fmt_spec.width.is_none());
	    assert!(fmt_spec.percision.is_none());
	    assert_eq!(fmt_spec.kind, Type::None);
	    true
	},
	_ => false,
    });
}

#[test]
fn all_in_one() {
    let formatstr = parse_fmt_str("Hello, {argument:-^+#0wide$.*X?}!").unwrap();
    assert_eq!(formatstr.text, vec!["Hello, ", "!"]);
    assert_eq!(formatstr.maybe_fmt.len(), 1);
    assert!(match &formatstr.maybe_fmt[0] {
	PossibleFormatSlot::FormatSlot(fmt) => {
	    assert!(match fmt.arg.as_ref().unwrap() {
		Argument::Identifier(str) => {assert_eq!(str, "argument"); true},
		_ => false
	    });
	    assert!(fmt.fmt_spec.is_some());
	    let fmt_spec = fmt.fmt_spec.as_ref().unwrap();
	    assert_eq!(fmt_spec.fill.as_ref().unwrap(), &'-');
	    assert_eq!(fmt_spec.align.as_ref().unwrap(), &Align::Center);
	    assert_eq!(fmt_spec.sign.as_ref().unwrap(), &Sign::Positive);
	    assert!(fmt_spec.alternate);
	    assert!(fmt_spec.pad_with_zeros);
	    assert!(match fmt_spec.width.as_ref().unwrap() {
		Count::Parameter(param) => {
		    match param {
			Argument::Identifier(wide) => wide == "wide",
			_ => false,
		    }
		}
		_ => false,
	    });
	    assert_eq!(fmt_spec.percision.as_ref().unwrap(), &Percision::SpecifiedPercision);
	    assert_eq!(fmt_spec.kind, Type::DebugUpperHex);
	    true
	},
	_ => false,
    });
}
