use {
    alpm::{
        alpm_sys::*, Alpm, Event, EventType, HookWhen, LogLevel, PackageOperation, Progress,
        Question,
    },
    dialoguer::{Confirm, Select},
    indicatif::{ProgressBar, ProgressStyle},
    std::{
        convert::TryInto,
        ffi::CStr,
        mem::transmute,
        os::raw::{c_char, c_int, c_void},
        ptr,
    },
};

static mut QUESTION_CALLBACK: Option<QuestionCallback> = None;
static mut LOG_CALLBACK: Option<LogCallback> = None;
static mut DL_CALLBACK: Option<DlCallback> = None;
static mut EVENT_CALLBACK: Option<EventCallback> = None;
static mut PROGRESS_CALLBACK: Option<ProgressCallback> = None;

static mut ALPM_HANDLE: *mut alpm_handle_t = ptr::null_mut();

pub fn init(handle: &Alpm) {
    unsafe {
        QUESTION_CALLBACK = Some(QuestionCallback::new());
        LOG_CALLBACK = Some(LogCallback::new());
        DL_CALLBACK = Some(DlCallback::new(handle.syncdbs().count()));
        EVENT_CALLBACK = Some(EventCallback::new());
        PROGRESS_CALLBACK = Some(ProgressCallback::new());

        ALPM_HANDLE = handle.as_alpm_handle_t();
    }
}

pub struct QuestionCallback;

impl QuestionCallback {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&mut self, question: &mut Question) {
        match question {
            Question::InstallIgnorepkg(question) => question.set_install(
                Confirm::new()
                    .with_prompt(format!(
                        "{} is in IgnorePkg. Install anyway?",
                        question.pkg().name()
                    ))
                    .interact()
                    .unwrap(),
            ),
            Question::Replace(question) => question.set_replace(
                Confirm::new()
                    .with_prompt(format!(
                        "Replace {} with {}/{}",
                        question.oldpkg().name(),
                        question.newdb().name(),
                        question.newpkg().name()
                    ))
                    .interact()
                    .unwrap(),
            ),
            Question::Conflict(question) => question.set_remove(
                Confirm::new()
                    .default(false)
                    .with_prompt(format!(
                        "{} and {} are in conflict. Remove {}?",
                        question.conflict().package1(),
                        question.conflict().package2(),
                        question.conflict().package1()
                    ))
                    .interact()
                    .unwrap(),
            ),
            Question::Corrupted(question) => question.set_remove(
                Confirm::new()
                    .with_prompt(format!(
                        "File {} is corrupted ({}). Remove it?",
                        question.filepath(),
                        question.reason()
                    ))
                    .interact()
                    .unwrap(),
            ),
            Question::RemovePkgs(question) => {
                println!(
                    "The following package[s] cannot be upgraded due to unresolvable dependencies:"
                );
                for pkg in question.packages() {
                    println!("{}", pkg.name());
                }

                question.set_skip(
                    Confirm::new()
                        .default(false)
                        .with_prompt("Do you want to skip the above package for this upgrade?")
                        .interact()
                        .unwrap(),
                );
            }
            Question::SelectProvider(question) => {
                question.set_index(
                    Select::new()
                        .with_prompt(format!(
                            "There are {} providers available for {}:",
                            question.providers().count(),
                            question.depend()
                        ))
                        .items(
                            question
                                .providers()
                                .map(|pkg| pkg.name().to_string())
                                .collect::<Vec<String>>()
                                .as_slice(),
                        )
                        .interact()
                        .unwrap()
                        .try_into()
                        .unwrap(),
                );
            }
            Question::ImportKey(question) => question.set_import(
                Confirm::new()
                    .with_prompt(format!(
                        "Import PGP key {} \"{}\"?",
                        question.key().fingerprint(),
                        question.key().uid()
                    ))
                    .interact()
                    .unwrap(),
            ),
        };
    }

    pub fn register() {
        unsafe extern "C" fn c_questioncb(question: *mut alpm_question_t) {
            let mut question = Question::new(ALPM_HANDLE, question);
            QUESTION_CALLBACK.as_mut().unwrap().update(&mut question);
        }

        unsafe { alpm_option_set_questioncb(ALPM_HANDLE, Some(c_questioncb)) };
    }
}

pub struct LogCallback;

impl LogCallback {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&self, level: LogLevel, message: &str) {
        let message = message.trim_end_matches('\n');
        match level {
            LogLevel::FUNCTION => log::trace!("{}", message),
            LogLevel::DEBUG => log::debug!("{}", message),
            LogLevel::WARNING => log::warn!("{}", message),
            LogLevel::ERROR => log::error!("{}", message),
            _ => unreachable!(),
        }
    }

    pub fn register() {
        extern "C" {
            fn vasprintf(
                str: *const *mut c_char,
                fmt: *const c_char,
                args: *mut __va_list_tag,
            ) -> c_int;
            fn free(ptr: *mut c_void);
        }

        unsafe extern "C" fn c_logcb(
            level: alpm_loglevel_t,
            fmt: *const c_char,
            args: *mut __va_list_tag,
        ) {
            let buff = ptr::null_mut();
            let n = vasprintf(&buff, fmt, args);
            if n != -1 {
                let s = CStr::from_ptr(buff);
                let level = LogLevel::from_bits(level).unwrap();
                LOG_CALLBACK
                    .as_mut()
                    .unwrap()
                    .update(level, &s.to_string_lossy());
                free(buff as *mut c_void);
            }
        }

        unsafe { alpm_option_set_logcb(ALPM_HANDLE, Some(c_logcb)) };
    }
}

pub struct DlCallback {
    bar: ProgressBar,
    db_count: usize,
    current_db: usize,
}

impl DlCallback {
    pub fn new(db_count: usize) -> Self {
        let bar = ProgressBar::new(0);
        bar.set_style(ProgressStyle::default_bar()
        .template("{prefix} Syncing {msg} {bytes}/{total_bytes} {bytes_per_sec} {eta} [{wide_bar:.cyan/blue}] {percent}%")
        .progress_chars("-> "));
        Self {
            bar,
            db_count,
            current_db: 0,
        }
    }
    pub fn update(&mut self, filename: &str, transferer: u64, total: u64) {
        if transferer != 0 && total != 0 {
            if transferer == total {
                if self.db_count == self.current_db {
                    self.bar.finish_and_clear();
                } else {
                    self.bar.finish();
                }
                self.bar.println(format!("  Synced {}", filename));
                return;
            }
            if self.bar.length() != total {
                self.bar.reset();
                self.bar.set_length(total);
                self.bar.set_message(filename);
                self.current_db += 1;
                self.bar
                    .set_prefix(format!("({}/{})", self.current_db, self.db_count).as_str());
            }
            self.bar.set_position(transferer);
        } else {
            self.bar.tick();
        }
    }

    pub fn register() {
        unsafe extern "C" fn c_dlcb(filename: *const c_char, xfered: off_t, total: off_t) {
            let filename = CStr::from_ptr(filename);
            let filename = filename.to_str().unwrap();
            DL_CALLBACK
                .as_mut()
                .unwrap()
                .update(&filename, xfered as u64, total as u64);
        }

        unsafe { alpm_option_set_dlcb(ALPM_HANDLE, Some(c_dlcb)) };
    }
}

pub struct EventCallback;

impl EventCallback {
    pub fn new() -> Self {
        Self {}
    }
    pub fn update(&mut self, event: &Event) {
        match event {
            Event::Hook(event) => match event.when() {
                HookWhen::PreTransaction => println!("Running pre-transaction hooks..."),
                HookWhen::PostTransaction => println!("Running post-transaction hooks..."),
            },
            Event::HookRun(event) => println!(
                "{:02}/{:02} {}",
                event.position(),
                event.total(),
                event.desc()
            ),
            Event::Other(event_type) => match event_type {
                EventType::CheckDepsStart => println!("checking dependencies..."),
                EventType::ResolveDepsStart => println!("resolving dependencies..."),
                EventType::InterConflictsStart => println!("looking for conflicting packages..."),
                EventType::TransactionStart => println!("Processing package changes..."),
                EventType::KeyDownloadStart => println!("downloading required keys"),
                EventType::PkgDownloadStart => println!("Retrieving packages..."),
                _ => {}
            },
            Event::PackageOperation(event) => match event.operation() {
                PackageOperation::Install(op) => todo!("Display optdepends"),
                PackageOperation::Upgrade(new, old) => todo!("Display new optdepends"),
                PackageOperation::Downgrade(new, old) => todo!("Display new optdepends"),
                PackageOperation::Reinstall(_, _) => {}
                PackageOperation::Remove(_) => {}
            },
            Event::ScriptletInfo(event) => print!("{}", event.line()),
            Event::OptDepRemoval(event) => println!(
                "{} optionally requires {}",
                event.pkg().name(),
                event.optdep()
            ),
            Event::DatabaseMissing(event) => println!(
                "database file for '{}' is missing (use '[S|F]y' to download)",
                event.dbname()
            ),
            Event::PacnewCreated(event) => todo!("Show pacnew and accumulate them"),
            Event::PacsaveCreated(event) => todo!("Show pacsave and accumulate them"),
            Event::PkgDownload(_) => {}
        }
    }

    pub fn register() {
        unsafe extern "C" fn c_eventcb(event: *mut alpm_event_t) {
            let event = Event::new(ALPM_HANDLE, event);
            EVENT_CALLBACK.as_mut().unwrap().update(&event);
        }

        unsafe { alpm_option_set_eventcb(ALPM_HANDLE, Some(c_eventcb)) };
    }
}

pub struct ProgressCallback {
    bar: ProgressBar,
    last_step: Option<Progress>,
}

impl ProgressCallback {
    pub fn new() -> Self {
        let bar = ProgressBar::new(100);
        bar.set_style(
            ProgressStyle::default_bar()
                .template("{prefix} {msg} [{wide_bar:.cyan/blue}] {percent}%")
                .progress_chars("-> "),
        );
        Self {
            bar,
            last_step: None,
        }
    }
    pub fn update(
        &mut self,
        progress: Progress,
        pkgname: &str,
        percent: i32,
        howmany: usize,
        current: usize,
    ) {
        if percent != 0 {
            if self.last_step != Some(progress) {
                self.bar.reset();
                self.last_step = Some(progress);
            }
            if percent == 100 {
                self.bar.finish();
            }
            self.bar.set_position(percent.try_into().unwrap());
            self.bar
                .set_prefix(format!("({}/{})", current, howmany).as_str());
            match progress {
                Progress::AddStart => {
                    self.bar
                        .set_message(format!("installing {}", pkgname).as_str());
                    if percent == 100 {
                        self.bar.println(format!("Installed {}", pkgname).as_str());
                    }
                }
                Progress::UpgradeStart => {
                    self.bar
                        .set_message(format!("upgrading {}", pkgname).as_str());
                    if percent == 100 {
                        self.bar.println(format!("Upgraded {}", pkgname).as_str());
                    }
                }
                Progress::DowngradeStart => {
                    self.bar
                        .set_message(format!("downgrading {}", pkgname).as_str());
                    if percent == 100 {
                        self.bar.println(format!("Downgraded {}", pkgname).as_str());
                    }
                }
                Progress::ReinstallStart => {
                    self.bar
                        .set_message(format!("reinstalling {}", pkgname).as_str());
                    if percent == 100 {
                        self.bar
                            .println(format!("Reinstalled {}", pkgname).as_str());
                    }
                }
                Progress::RemoveStart => {
                    self.bar
                        .set_message(format!("removing {}", pkgname).as_str());
                    if percent == 100 {
                        self.bar.println(format!("Removed {}", pkgname).as_str());
                    }
                }
                Progress::ConflictsStart => {
                    self.bar.set_message("checking for file conflicts");
                    if percent == 100 && howmany == current {
                        self.bar.println("Checked for file conflicts");
                    }
                }
                Progress::DiskspaceStart => {
                    self.bar.set_message("checking available disk space");
                    if percent == 100 && howmany == current {
                        self.bar.println("Checked available disk space");
                    }
                }
                Progress::IntegrityStart => {
                    self.bar.set_message("checking package integrity");
                    if percent == 100 && howmany == current {
                        self.bar.println("Checked package integrity");
                    }
                }
                Progress::KeyringStart => {
                    self.bar.set_message("checking keys in keyring");
                    if percent == 100 && howmany == current {
                        self.bar.println("Checked keys in keyring");
                    }
                }
                Progress::LoadStart => {
                    self.bar.set_message("loading package files");
                    if percent == 100 && howmany == current {
                        self.bar.println("Loaded package files");
                    }
                }
            }
        } else {
            self.bar.tick();
        }
    }

    pub fn register() {
        unsafe extern "C" fn c_progresscb(
            progress: alpm_progress_t,
            pkgname: *const c_char,
            percent: c_int,
            howmany: usize,
            current: usize,
        ) {
            let pkgname = CStr::from_ptr(pkgname);
            let pkgname = pkgname.to_str().unwrap();
            let progress = transmute::<alpm_progress_t, Progress>(progress);
            PROGRESS_CALLBACK.as_mut().unwrap().update(
                progress,
                &pkgname,
                percent as i32,
                howmany,
                current,
            );
        }

        unsafe { alpm_option_set_progresscb(ALPM_HANDLE, Some(c_progresscb)) };
    }
}
