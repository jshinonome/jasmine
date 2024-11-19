use anyhow::Result;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum JError {
    #[error("{0}")]
    Err(String),

    #[error("Eval Err: {0}")]
    EvalErr(String),

    #[error("Err: {0}")]
    RaiseErr(String),

    #[error("Invalid Handle err")]
    InvalidHandleErr(),

    #[error("Parser err: {0}")]
    ParserErr(String),

    #[error("Failed to refer {0} from {1}")]
    MismatchedTypeErr(String, String),

    #[error("Unsupported unary op '{0}' for '{1}'")]
    UnsupportedUnaryOpErr(String, String),

    #[error("Unsupported binary op '{0}' between '{1}' and '{2}'")]
    UnsupportedBinaryOpErr(String, String, String),

    #[error("Unsupported sql J type '{0}'")]
    UnsupportedSqlJTypeErr(String),

    #[error("Unsupported sql unary op '{0}'")]
    UnsupportedSqlUnaryOpErr(String),

    #[error("Unsupported sql binary op '{0}'")]
    UnsupportedSqlBinaryOpErr(String),

    #[error("Not serializable '{0}'")]
    NotSerializableErr(String),

    #[error("Not series")]
    NotSeriesErr(),

    #[error("Not yet implemented '{0}'")]
    NotYetImplemented(String),

    #[error("Expect {0} argument(s), {1} given")]
    MismatchedArgNumErr(usize, usize),

    #[error("Expect {0} argument(s) function, {1} argument(s) function given")]
    MismatchedArgNumFnErr(usize, usize),

    #[error("Expect '{0}' for '{1}' argument , got '{2}'")]
    MismatchedArgTypeErr(String, usize, String),

    #[error("Name '{0}' is not defined")]
    NameErr(String),

    #[error("Length error '{0}' vs '{1}'")]
    MismatchedLengthErr(usize, usize),

    #[error("Forbidden '{0}' keyword")]
    ForbiddenKeywordErr(String),

    #[error("OS Err: '{0}'")]
    OsErr(String),

    #[error("Failed to read lock '{0}'")]
    ReadLockErr(String),

    #[error("Failed to write lock '{0}'")]
    WriteLockErr(String),
}

pub type JResult<J> = Result<J, JError>;

pub fn trace(source: &str, path: &str, pos: usize, msg: &str) -> String {
    let mut start = 0;
    let mut r = 1;
    let mut c = 1;
    let mut chars = source.chars().peekable();
    let mut i = 0;
    while i < pos {
        match chars.next() {
            Some('\r') => {
                if let Some(&'\n') = chars.peek() {
                    chars.next();
                    i += 2;
                    r += 1;
                    c = 1;
                    start = i;
                } else {
                    i += 1;
                    c += 1;
                }
            }
            Some('\n') => {
                i += 1;
                r += 1;
                c = 1;
                start = i;
            }
            Some(ch) => {
                i += ch.len_utf8();
                c += 1;
            }
            None => unreachable!(),
        }
    }
    let end = match &source[pos..].chars().position(|c| c == '\n' || c == '\r') {
        Some(i) => pos + i,
        None => source.len(),
    };
    let line = &source[start..end];
    let underline = " ".repeat(c - 1) + "^";

    format!(
        "--> {path}{r}:{c}\n\
        \n\
        {line}\n\
        {underline}\n\
        \n\
        = {msg}"
    )
}

#[test]
fn display_trace() {
    let input = "1+1;\r\n1;\n`a + 1;";

    assert_eq!(
        trace(&input, "", 12, "type"),
        ["--> 3:4", "", "`a + 1;", "   ^", "", "= type"].join("\n")
    );
}
