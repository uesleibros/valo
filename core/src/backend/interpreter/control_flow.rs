use crate::ResumeTarget;
use crate::runtime::Value;

#[derive(Debug, Clone)]
pub(crate) enum ControlFlow {
    Continue,
    Return(Value),
    ExitSub,
    ExitFunction,
    ExitProperty,
    ExitFor,
    ExitWhile,
    ExitDo,
    ContinueFor,
    ContinueWhile,
    ContinueDo,
    GoTo(String),
    Resume(ResumeTarget),
    Terminate,
}
