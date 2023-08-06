pub enum Request {
    Initialize,
    Run,
    Shutdown,
}

pub enum Notif {
    Initialized,
    Message { kind: MessageKind, message: String },
    Completed,
    Exit,
}

pub enum MessageKind {
    Error,
    Warning,
    Note,
    Log,
}
