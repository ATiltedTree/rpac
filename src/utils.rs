use alpm::Alpm;

use crate::callbacks::*;

pub fn register_cbs(handle: &Alpm) {
    init(handle);
    QuestionCallback::register();
    LogCallback::register();
    DlCallback::register();
    EventCallback::register();
    ProgressCallback::register();
}
