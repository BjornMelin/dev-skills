use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use codex_dev_core::{
    CapsuleStatus, CheckRecord, PrEvidence, ReviewThreadSummary, StatusResult, Subagents,
    ValidationResult, Verification, capsule_status, read_json, render_pr_label, validate_capsule,
};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::backend::{Backend, TestBackend};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};
use ratatui::{Frame, Terminal};

#[derive(Parser, Debug)]
#[command(name = "codex-dev-tui")]
#[command(about = "Terminal workbench for codex-dev task capsules")]
/// Command-line options for the `codex-dev-tui` binary.
pub struct Cli {
    #[arg(
        long,
        value_name = "CAPSULE_DIR",
        help = "Open a single capsule immediately instead of the dashboard"
    )]
    capsule: Option<PathBuf>,
    #[arg(
        long,
        value_name = "TASKS_ROOT",
        default_value = ".codex/tasks",
        help = "Root directory scanned by dashboard mode"
    )]
    root: PathBuf,
    #[arg(
        long,
        help = "Render one deterministic frame to stdout instead of opening a terminal"
    )]
    render_once: bool,
    #[arg(long, default_value_t = 100, help = "Render-once width")]
    width: u16,
    #[arg(long, default_value_t = 30, help = "Render-once height")]
    height: u16,
    #[arg(
        long,
        default_value_t = 250,
        help = "Interactive poll interval in milliseconds"
    )]
    tick_ms: u64,
}

/// Parse CLI arguments and run either the interactive TUI or deterministic render mode.
pub fn run() -> Result<()> {
    let cli = Cli::parse();
    if cli.render_once {
        let result = render_once_for_cli(cli.capsule.as_deref(), &cli.root, cli.width, cli.height)?;
        print!("{}", result.output);
        if !result.valid {
            anyhow::bail!("invalid capsule; see render output for validation details");
        }
        return Ok(());
    }

    run_interactive(
        cli.capsule.as_deref(),
        &cli.root,
        interactive_tick_rate(cli.tick_ms)?,
    )
    .map_err(|error| match cli.capsule.as_deref() {
        Some(capsule) => sanitized_cli_error(error, capsule),
        None => sanitized_path_error(error, &cli.root, "<tasks-root>"),
    })
}

/// Open the interactive terminal UI for the dashboard or one local `codex-dev` capsule.
pub fn run_interactive(
    capsule_path: Option<&Path>,
    dashboard_root: &Path,
    tick_rate: Duration,
) -> Result<()> {
    let mut terminal = ratatui::init();
    let mut restore_guard = RestoreGuard::new(ratatui::restore);
    let mut state = AppState::load(capsule_path, dashboard_root)?;
    let result = run_app(
        &mut terminal,
        &mut state,
        &mut CrosstermEvents { tick_rate },
    );
    restore_guard.restore_now();
    result
}

/// Drive the render/event loop until the event source requests exit or errors.
pub fn run_app<B, E>(terminal: &mut Terminal<B>, state: &mut AppState, events: &mut E) -> Result<()>
where
    B: Backend,
    B::Error: std::error::Error + Send + Sync + 'static,
    E: EventSource,
{
    loop {
        terminal.draw(|frame| render_app(frame, state))?;
        match events.next_event()? {
            Some(WorkbenchEvent::Quit) => return Ok(()),
            Some(WorkbenchEvent::NextPanel) => state.next_panel(),
            Some(WorkbenchEvent::PreviousPanel) => state.previous_panel(),
            Some(WorkbenchEvent::NextItem) => state.next_item(),
            Some(WorkbenchEvent::PreviousItem) => state.previous_item(),
            Some(WorkbenchEvent::OpenSelected) => state.open_selected()?,
            Some(WorkbenchEvent::Dashboard) => state.show_dashboard(),
            Some(WorkbenchEvent::CycleFilter) => state.cycle_filter(),
            Some(WorkbenchEvent::CycleSort) => state.cycle_sort(),
            Some(WorkbenchEvent::Refresh) => state.refresh(),
            None => {}
        }
    }
}

/// Source of high-level workbench events for the application loop.
pub trait EventSource {
    /// Return the next event, or `None` when the loop should render again after a tick.
    fn next_event(&mut self) -> Result<Option<WorkbenchEvent>>;
}

/// Crossterm-backed event source for interactive terminal sessions.
pub struct CrosstermEvents {
    tick_rate: Duration,
}

impl EventSource for CrosstermEvents {
    fn next_event(&mut self) -> Result<Option<WorkbenchEvent>> {
        if !event::poll(self.tick_rate)? {
            return Ok(None);
        }

        match event::read()? {
            Event::Key(key) if key.kind != KeyEventKind::Release => Ok(map_key(key)),
            _ => Ok(None),
        }
    }
}

fn map_key(key: KeyEvent) -> Option<WorkbenchEvent> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => Some(WorkbenchEvent::Quit),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(WorkbenchEvent::Quit)
        }
        KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => Some(WorkbenchEvent::NextPanel),
        KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') => {
            Some(WorkbenchEvent::PreviousPanel)
        }
        KeyCode::Down | KeyCode::Char('j') => Some(WorkbenchEvent::NextItem),
        KeyCode::Up | KeyCode::Char('k') => Some(WorkbenchEvent::PreviousItem),
        KeyCode::Enter => Some(WorkbenchEvent::OpenSelected),
        KeyCode::Backspace | KeyCode::Char('b') => Some(WorkbenchEvent::Dashboard),
        KeyCode::Char('f') => Some(WorkbenchEvent::CycleFilter),
        KeyCode::Char('s') => Some(WorkbenchEvent::CycleSort),
        KeyCode::Char('r') => Some(WorkbenchEvent::Refresh),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// High-level actions the workbench event loop understands.
pub enum WorkbenchEvent {
    Quit,
    NextPanel,
    PreviousPanel,
    NextItem,
    PreviousItem,
    OpenSelected,
    Dashboard,
    CycleFilter,
    CycleSort,
    Refresh,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Top-level panel currently rendered in the workbench detail area.
pub enum Panel {
    Overview,
    Validation,
    Pr,
    Help,
}

impl Panel {
    fn next(self) -> Self {
        match self {
            Self::Overview => Self::Validation,
            Self::Validation => Self::Pr,
            Self::Pr => Self::Help,
            Self::Help => Self::Overview,
        }
    }

    fn previous(self) -> Self {
        match self {
            Self::Overview => Self::Help,
            Self::Validation => Self::Overview,
            Self::Pr => Self::Validation,
            Self::Help => Self::Pr,
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Validation => "Validation",
            Self::Pr => "PR",
            Self::Help => "Help",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Dashboard capsule filter used by multi-capsule navigation.
pub enum DashboardFilter {
    All,
    Active,
    InReview,
    Invalid,
    HasPr,
}

impl DashboardFilter {
    fn next(self) -> Self {
        match self {
            Self::All => Self::Active,
            Self::Active => Self::InReview,
            Self::InReview => Self::Invalid,
            Self::Invalid => Self::HasPr,
            Self::HasPr => Self::All,
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Active => "active",
            Self::InReview => "review",
            Self::Invalid => "invalid",
            Self::HasPr => "has-pr",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Dashboard sorting mode used for recent-capsule scanning.
pub enum DashboardSort {
    UpdatedDesc,
    TitleAsc,
    StatusAsc,
}

impl DashboardSort {
    fn next(self) -> Self {
        match self {
            Self::UpdatedDesc => Self::TitleAsc,
            Self::TitleAsc => Self::StatusAsc,
            Self::StatusAsc => Self::UpdatedDesc,
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::UpdatedDesc => "updated",
            Self::TitleAsc => "title",
            Self::StatusAsc => "status",
        }
    }
}

#[derive(Debug)]
/// Single row in the dashboard capsule list.
pub struct DashboardCapsule {
    pub path: PathBuf,
    pub display_title: String,
    pub status_label: String,
    pub updated_label: String,
    pub validation: ValidationResult,
    pub capsule: Option<StatusResult>,
    pub verification: Option<Verification>,
    pub subagents: Option<Subagents>,
    pub pr: Option<PrEvidence>,
    pub diagnostics: Vec<String>,
}

impl DashboardCapsule {
    fn load(path: PathBuf) -> Self {
        let validation = match validate_capsule(&path) {
            Ok(validation) => validation,
            Err(error) => ValidationResult {
                path: path.clone(),
                valid: false,
                errors: vec![format!("{error:#}")],
            },
        };
        if !validation.valid {
            let display_title = fallback_dashboard_title(&path);
            return Self {
                path,
                display_title,
                status_label: "invalid".to_string(),
                updated_label: "unknown".to_string(),
                diagnostics: validation.errors.clone(),
                validation,
                capsule: None,
                verification: None,
                subagents: None,
                pr: None,
            };
        }

        let capsule = load_optional_contract(&path, "capsule", || capsule_status(&path));
        let verification = load_optional_contract(&path, "verification.json", || {
            read_json(&path.join("verification.json"))
        });
        let subagents = load_optional_contract(&path, "subagents.json", || {
            read_json(&path.join("subagents.json"))
        });
        let pr = load_optional_contract(&path, "pr.json", || read_json(&path.join("pr.json")));
        let mut diagnostics = Vec::new();
        diagnostics.extend(capsule.1);
        diagnostics.extend(verification.1);
        diagnostics.extend(subagents.1);
        diagnostics.extend(pr.1);
        let display_title = capsule
            .0
            .as_ref()
            .map(|capsule| capsule.title.clone())
            .unwrap_or_else(|| fallback_dashboard_title(&path));
        let status_label = capsule
            .0
            .as_ref()
            .map(|capsule| capsule.status.to_string())
            .unwrap_or_else(|| "invalid".to_string());
        let updated_label = capsule
            .0
            .as_ref()
            .map(|capsule| capsule.updated_at.to_rfc3339())
            .unwrap_or_else(|| "unknown".to_string());

        Self {
            path,
            display_title,
            status_label,
            updated_label,
            validation,
            capsule: capsule.0,
            verification: verification.0,
            subagents: subagents.0,
            pr: pr.0,
            diagnostics,
        }
    }

    fn title(&self) -> &str {
        &self.display_title
    }

    fn status_label(&self) -> &str {
        &self.status_label
    }

    fn updated_label(&self) -> &str {
        &self.updated_label
    }

    fn matches_filter(&self, filter: DashboardFilter) -> bool {
        match filter {
            DashboardFilter::All => true,
            DashboardFilter::Invalid => !self.validation.valid,
            DashboardFilter::Active => matches!(
                self.capsule.as_ref().map(|capsule| &capsule.status),
                Some(CapsuleStatus::Active | CapsuleStatus::Blocked | CapsuleStatus::ReadyForPr)
            ),
            DashboardFilter::InReview => matches!(
                self.capsule.as_ref().map(|capsule| &capsule.status),
                Some(CapsuleStatus::InReview)
            ),
            DashboardFilter::HasPr => self
                .pr
                .as_ref()
                .is_some_and(|pr| pr.number.is_some() || pr.state != "not_created"),
        }
    }
}

fn fallback_dashboard_title(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(str::to_string)
        .unwrap_or_else(|| "<unknown>".to_string())
}

#[derive(Debug)]
/// Read-only multi-capsule dashboard state.
pub struct DashboardState {
    pub root: PathBuf,
    pub capsules: Vec<DashboardCapsule>,
    pub selected: usize,
    pub filter: DashboardFilter,
    pub sort: DashboardSort,
    pub diagnostics: Vec<String>,
}

impl DashboardState {
    pub fn load(root: impl AsRef<Path>) -> Self {
        Self::load_with_view(root, DashboardFilter::All, DashboardSort::UpdatedDesc, None)
    }

    fn load_with_view(
        root: impl AsRef<Path>,
        filter: DashboardFilter,
        sort: DashboardSort,
        selected_path: Option<&Path>,
    ) -> Self {
        let root = root.as_ref().to_path_buf();
        let mut diagnostics = Vec::new();
        let mut capsules = Vec::new();
        match fs::read_dir(&root) {
            Ok(entries) => {
                for entry in entries {
                    match entry {
                        Ok(entry) => {
                            let path = entry.path();
                            match entry.file_type() {
                                Ok(file_type) if file_type.is_dir() => {
                                    capsules.push(DashboardCapsule::load(path));
                                }
                                Ok(_) => {}
                                Err(error) => diagnostics
                                    .push(format!("failed to inspect {}: {error}", path.display())),
                            }
                        }
                        Err(error) => {
                            diagnostics.push(format!(
                                "failed to read entry under {}: {error}",
                                root.display()
                            ));
                        }
                    }
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                diagnostics.push(format!("dashboard root does not exist: {}", root.display()));
            }
            Err(error) => diagnostics.push(format!(
                "failed to read dashboard root {}: {error}",
                root.display()
            )),
        }
        sort_dashboard_capsules(&mut capsules, sort);
        let mut state = Self {
            root,
            capsules,
            selected: 0,
            filter,
            sort,
            diagnostics,
        };
        state.restore_selection(selected_path);
        state
    }

    fn refresh(&mut self) {
        let selected_path = self.selected_capsule().map(|capsule| capsule.path.clone());
        *self = Self::load_with_view(
            self.root.clone(),
            self.filter,
            self.sort,
            selected_path.as_deref(),
        );
    }

    fn selected_capsule(&self) -> Option<&DashboardCapsule> {
        self.filtered_indices()
            .get(self.selected)
            .and_then(|index| self.capsules.get(*index))
    }

    fn filtered_indices(&self) -> Vec<usize> {
        self.capsules
            .iter()
            .enumerate()
            .filter_map(|(index, capsule)| capsule.matches_filter(self.filter).then_some(index))
            .collect()
    }

    fn next_item(&mut self) {
        let len = self.filtered_indices().len();
        if len > 0 {
            self.selected = (self.selected + 1).min(len - 1);
        }
    }

    fn previous_item(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    fn cycle_filter(&mut self) {
        self.filter = self.filter.next();
        self.selected = 0;
        self.clamp_selection();
    }

    fn cycle_sort(&mut self) {
        let selected_path = self.selected_capsule().map(|capsule| capsule.path.clone());
        self.sort = self.sort.next();
        sort_dashboard_capsules(&mut self.capsules, self.sort);
        self.restore_selection(selected_path.as_deref());
    }

    fn restore_selection(&mut self, selected_path: Option<&Path>) {
        self.selected = selected_path
            .and_then(|path| {
                self.filtered_indices()
                    .iter()
                    .position(|index| self.capsules[*index].path == path)
            })
            .unwrap_or(0);
        self.clamp_selection();
    }

    fn clamp_selection(&mut self) {
        let len = self.filtered_indices().len();
        if len == 0 {
            self.selected = 0;
        } else if self.selected >= len {
            self.selected = len - 1;
        }
    }
}

#[derive(Debug)]
/// Top-level TUI state. Dashboard is always retained so detail mode can return to it.
pub struct AppState {
    pub dashboard: DashboardState,
    pub capsule: Option<WorkbenchState>,
}

impl AppState {
    pub fn load(capsule_path: Option<&Path>, dashboard_root: &Path) -> Result<Self> {
        Ok(Self {
            dashboard: DashboardState::load(dashboard_root),
            capsule: capsule_path.map(WorkbenchState::load).transpose()?,
        })
    }

    fn refresh(&mut self) {
        if let Some(capsule) = &mut self.capsule {
            capsule.refresh();
        } else {
            self.dashboard.refresh();
        }
    }

    fn next_panel(&mut self) {
        if let Some(capsule) = &mut self.capsule {
            capsule.next_panel();
        }
    }

    fn previous_panel(&mut self) {
        if let Some(capsule) = &mut self.capsule {
            capsule.previous_panel();
        }
    }

    fn next_item(&mut self) {
        if self.capsule.is_none() {
            self.dashboard.next_item();
        }
    }

    fn previous_item(&mut self) {
        if self.capsule.is_none() {
            self.dashboard.previous_item();
        }
    }

    fn cycle_filter(&mut self) {
        if self.capsule.is_none() {
            self.dashboard.cycle_filter();
        }
    }

    fn cycle_sort(&mut self) {
        if self.capsule.is_none() {
            self.dashboard.cycle_sort();
        }
    }

    fn open_selected(&mut self) -> Result<()> {
        if self.capsule.is_none()
            && let Some(capsule) = self.dashboard.selected_capsule()
            && capsule.validation.valid
        {
            self.capsule = Some(WorkbenchState::load(&capsule.path)?);
        }
        Ok(())
    }

    fn show_dashboard(&mut self) {
        let selected_path = self
            .capsule
            .as_ref()
            .map(|capsule| capsule.capsule_path.clone());
        self.capsule = None;
        if let Some(selected_path) = selected_path {
            self.dashboard = DashboardState::load_with_view(
                self.dashboard.root.clone(),
                self.dashboard.filter,
                self.dashboard.sort,
                Some(&selected_path),
            );
        }
    }
}

fn load_optional_contract<T, F>(path: &Path, label: &str, load: F) -> (Option<T>, Vec<String>)
where
    F: FnOnce() -> Result<T>,
{
    match load() {
        Ok(value) => (Some(value), Vec::new()),
        Err(error) => (
            None,
            vec![format!(
                "{label}: {}",
                redact_path_text(&format!("{error:#}"), path)
            )],
        ),
    }
}

fn sort_dashboard_capsules(capsules: &mut [DashboardCapsule], sort: DashboardSort) {
    match sort {
        DashboardSort::UpdatedDesc => capsules.sort_by(|left, right| {
            right
                .capsule
                .as_ref()
                .map(|capsule| capsule.updated_at)
                .cmp(&left.capsule.as_ref().map(|capsule| capsule.updated_at))
                .then_with(|| left.title().cmp(right.title()))
                .then_with(|| left.path.as_os_str().cmp(right.path.as_os_str()))
        }),
        DashboardSort::TitleAsc => capsules.sort_by(|left, right| {
            left.title()
                .cmp(right.title())
                .then_with(|| left.path.as_os_str().cmp(right.path.as_os_str()))
        }),
        DashboardSort::StatusAsc => capsules.sort_by(|left, right| {
            left.status_label()
                .cmp(right.status_label())
                .then_with(|| left.title().cmp(right.title()))
                .then_with(|| left.path.as_os_str().cmp(right.path.as_os_str()))
        }),
    }
}

#[derive(Debug)]
/// View model loaded from a local `codex-dev` capsule directory.
pub struct WorkbenchState {
    pub capsule_path: PathBuf,
    pub validation: ValidationResult,
    pub capsule: Option<StatusResult>,
    pub verification: Option<Verification>,
    pub pr: Option<PrEvidence>,
    pub active_panel: Panel,
    pub last_error: Option<String>,
}

impl WorkbenchState {
    /// Validate and load the capsule contracts used by the TUI.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let validation = validate_capsule(&path)?;
        let (capsule, verification, pr) = if validation.valid {
            (
                Some(capsule_status(&path)?),
                read_json(&path.join("verification.json")).ok(),
                read_json(&path.join("pr.json")).ok(),
            )
        } else {
            (None, None, None)
        };

        Ok(Self {
            capsule_path: path,
            validation,
            capsule,
            verification,
            pr,
            active_panel: Panel::Overview,
            last_error: None,
        })
    }

    /// Reload capsule contracts while preserving the active panel.
    pub fn refresh(&mut self) {
        let active_panel = self.active_panel;
        match Self::load(&self.capsule_path) {
            Ok(mut next) => {
                next.active_panel = active_panel;
                *self = next;
            }
            Err(error) => {
                let message = redact_path_text(&format!("{error:#}"), &self.capsule_path);
                self.replace_contracts_with_error(message);
            }
        }
    }

    /// Advance to the next panel, wrapping at the end.
    pub fn next_panel(&mut self) {
        self.active_panel = self.active_panel.next();
    }

    /// Move to the previous panel, wrapping at the beginning.
    pub fn previous_panel(&mut self) {
        self.active_panel = self.active_panel.previous();
    }

    fn replace_contracts_with_error(&mut self, message: String) {
        self.validation = ValidationResult {
            path: self.capsule_path.clone(),
            valid: false,
            errors: vec![message.clone()],
        };
        self.capsule = None;
        self.verification = None;
        self.pr = None;
        self.last_error = Some(message);
    }
}

/// Restores terminal state exactly once on explicit restore or drop.
pub struct RestoreGuard<F>
where
    F: FnMut(),
{
    restore: F,
    armed: bool,
}

impl<F> RestoreGuard<F>
where
    F: FnMut(),
{
    /// Create an armed restore guard from a cleanup callback.
    pub fn new(restore: F) -> Self {
        Self {
            restore,
            armed: true,
        }
    }

    /// Run the cleanup callback immediately if it has not already run.
    pub fn restore_now(&mut self) {
        if self.armed {
            (self.restore)();
            self.armed = false;
        }
    }
}

impl<F> Drop for RestoreGuard<F>
where
    F: FnMut(),
{
    fn drop(&mut self) {
        self.restore_now();
    }
}

/// Render the full workbench frame for the supplied state.
pub fn render_app(frame: &mut Frame<'_>, state: &AppState) {
    if let Some(capsule) = &state.capsule {
        render(frame, capsule);
    } else {
        render_dashboard(frame, &state.dashboard);
    }
}

/// Render the multi-capsule dashboard frame.
pub fn render_dashboard(frame: &mut Frame<'_>, state: &DashboardState) {
    let root = Block::default()
        .title(" codex-dev dashboard ")
        .borders(Borders::ALL);
    let inner = root.inner(frame.area());
    frame.render_widget(root, frame.area());

    let [header, body, footer] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(8),
        Constraint::Length(3),
    ])
    .areas(inner);

    render_dashboard_header(frame, header, state);
    render_dashboard_body(frame, body, state);
    render_dashboard_footer(frame, footer, state);
}

fn render_dashboard_header(frame: &mut Frame<'_>, area: Rect, state: &DashboardState) {
    let visible = state.filtered_indices().len();
    let line = Line::from(vec![
        Span::styled(
            format!("root: {}", state.root.display()),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            format!("visible: {visible}/{}", state.capsules.len()),
            Style::default().fg(Color::Cyan),
        ),
        Span::raw("  "),
        Span::raw(format!("filter: {}", state.filter.title())),
        Span::raw("  "),
        Span::raw(format!("sort: {}", state.sort.title())),
    ]);
    frame.render_widget(
        Paragraph::new(line)
            .block(Block::default().title(" Dashboard ").borders(Borders::ALL))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_dashboard_body(frame: &mut Frame<'_>, area: Rect, state: &DashboardState) {
    let [left, right] =
        Layout::horizontal([Constraint::Percentage(48), Constraint::Percentage(52)]).areas(area);
    frame.render_widget(
        List::new(dashboard_items(
            state,
            left.height.saturating_sub(2) as usize,
        ))
        .block(Block::default().title(" Capsules ").borders(Borders::ALL)),
        left,
    );
    frame.render_widget(
        Paragraph::new(dashboard_detail_text(state))
            .block(Block::default().title(" Details ").borders(Borders::ALL))
            .wrap(Wrap { trim: true }),
        right,
    );
}

fn render_dashboard_footer(frame: &mut Frame<'_>, area: Rect, state: &DashboardState) {
    let message = if state.diagnostics.is_empty() {
        "up/down: select  enter: open  f: filter  s: sort  r: refresh  q/esc/ctrl-c: quit"
            .to_string()
    } else {
        format!(
            "diagnostics: {}  up/down: select  enter: open  f: filter  s: sort  r: refresh",
            state.diagnostics.len()
        )
    };
    frame.render_widget(
        Paragraph::new(message)
            .block(Block::default().title(" Keys ").borders(Borders::ALL))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn dashboard_items(state: &DashboardState, visible_rows: usize) -> Vec<ListItem<'static>> {
    let indices = state.filtered_indices();
    if indices.is_empty() {
        return vec![ListItem::new("no capsules match current filter")];
    }

    let visible_rows = visible_rows.max(1);
    let start = dashboard_window_start(state.selected, indices.len(), visible_rows);
    indices
        .iter()
        .skip(start)
        .take(visible_rows)
        .enumerate()
        .map(|(window_index, index)| {
            let visible_index = start + window_index;
            let capsule = &state.capsules[*index];
            let selected = if visible_index == state.selected {
                "> "
            } else {
                "  "
            };
            let style = if !capsule.validation.valid {
                Style::default().fg(Color::Red)
            } else {
                capsule
                    .capsule
                    .as_ref()
                    .map(capsule_status_style)
                    .unwrap_or_default()
            };
            ListItem::new(format!(
                "{selected}{} [{}] ev:{} pr:{} sa:{}",
                capsule.title(),
                capsule.status_label(),
                evidence_total(capsule),
                pr_summary(capsule),
                subagent_brief(capsule)
            ))
            .style(style)
        })
        .collect()
}

fn dashboard_window_start(selected: usize, len: usize, visible_rows: usize) -> usize {
    if len <= visible_rows {
        0
    } else {
        selected.saturating_add(1).saturating_sub(visible_rows)
    }
}

fn dashboard_detail_text(state: &DashboardState) -> String {
    let mut lines = Vec::new();
    if !state.diagnostics.is_empty() {
        lines.push("dashboard diagnostics:".to_string());
        lines.extend(state.diagnostics.iter().map(|diagnostic| {
            format!(
                "- {}",
                redact_path_text_with_placeholder(diagnostic, &state.root, "<tasks-root>")
            )
        }));
        lines.push(String::new());
    }

    let Some(capsule) = state.selected_capsule() else {
        lines.push("No capsule selected.".to_string());
        return lines.join("\n");
    };

    lines.push(format!("title: {}", capsule.title()));
    lines.push(format!(
        "path: {}",
        redact_path_text_with_placeholder(
            &capsule.path.display().to_string(),
            &state.root,
            "<tasks-root>",
        )
    ));
    lines.push(format!("validation: {}", validation_summary(capsule)));
    lines.push(format!("updated: {}", capsule.updated_label()));
    lines.push(format!("evidence: {}", evidence_total(capsule)));
    lines.push(format!("subagents: {}", subagent_summary(capsule)));
    lines.push(format!("pr: {}", pr_detail_summary(capsule)));
    if let Some(verification) = &capsule.verification {
        lines.push(format!(
            "validation gates: {}",
            gate_summary(&verification.required)
        ));
    }
    if !capsule.diagnostics.is_empty() {
        lines.push("capsule diagnostics:".to_string());
        lines.extend(
            capsule
                .diagnostics
                .iter()
                .map(|diagnostic| format!("- {}", redact_path_text(diagnostic, &capsule.path))),
        );
    }
    lines.join("\n")
}

fn validation_summary(capsule: &DashboardCapsule) -> String {
    if capsule.validation.valid {
        "valid".to_string()
    } else {
        format!("invalid ({} error(s))", capsule.validation.errors.len())
    }
}

fn evidence_total(capsule: &DashboardCapsule) -> u64 {
    capsule
        .capsule
        .as_ref()
        .map(|capsule| capsule.evidence.total)
        .unwrap_or(0)
}

fn subagent_summary(capsule: &DashboardCapsule) -> String {
    let Some(subagents) = &capsule.subagents else {
        return "none".to_string();
    };
    if subagents.batches.is_empty() {
        return "none".to_string();
    }
    let completed = subagents
        .batches
        .iter()
        .filter(|batch| batch.status == "completed")
        .count();
    format!("{completed}/{} completed", subagents.batches.len())
}

fn subagent_brief(capsule: &DashboardCapsule) -> String {
    let Some(subagents) = &capsule.subagents else {
        return "0/0".to_string();
    };
    let completed = subagents
        .batches
        .iter()
        .filter(|batch| batch.status == "completed")
        .count();
    format!("{completed}/{}", subagents.batches.len())
}

fn pr_summary(capsule: &DashboardCapsule) -> String {
    let Some(pr) = &capsule.pr else {
        return "none".to_string();
    };
    if let Some(number) = pr.number {
        format!("#{number}")
    } else if pr.state == "not_created" {
        "none".to_string()
    } else {
        pr.state.clone()
    }
}

fn pr_detail_summary(capsule: &DashboardCapsule) -> String {
    let Some(pr) = &capsule.pr else {
        return "none".to_string();
    };
    format!(
        "{}; checks:{}; unresolved:{}",
        render_pr_label(pr),
        pr.checks.len(),
        review_thread_unresolved_label(&pr.review_threads)
    )
}

fn review_thread_unresolved_label(summary: &ReviewThreadSummary) -> String {
    if summary.authoritative {
        summary.unresolved.to_string()
    } else {
        "not checked".to_string()
    }
}

fn gate_summary(gates: &[codex_dev_core::GateRecord]) -> String {
    if gates.is_empty() {
        return "none".to_string();
    }
    let passed = gates.iter().filter(|gate| gate.status == "passed").count();
    let failed = gates.iter().filter(|gate| gate.status == "failed").count();
    format!("{passed} passed, {failed} failed, {} total", gates.len())
}

fn capsule_status_style(capsule: &StatusResult) -> Style {
    match capsule.status {
        CapsuleStatus::Merged | CapsuleStatus::Closed => Style::default().fg(Color::Green),
        CapsuleStatus::Blocked => Style::default().fg(Color::Red),
        CapsuleStatus::ReadyForPr | CapsuleStatus::InReview => Style::default().fg(Color::Yellow),
        CapsuleStatus::Active => Style::default().fg(Color::Cyan),
    }
}

/// Render the full single-capsule workbench frame for the supplied state.
pub fn render(frame: &mut Frame<'_>, state: &WorkbenchState) {
    let root = Block::default()
        .title(" codex-dev workbench ")
        .borders(Borders::ALL);
    let inner = root.inner(frame.area());
    frame.render_widget(root, frame.area());

    let [header, body, footer] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(8),
        Constraint::Length(3),
    ])
    .areas(inner);

    render_header(frame, header, state);
    render_body(frame, body, state);
    render_footer(frame, footer, state);
}

fn render_header(frame: &mut Frame<'_>, area: Rect, state: &WorkbenchState) {
    let title = state
        .capsule
        .as_ref()
        .map(|capsule| capsule.title.as_str())
        .unwrap_or("Invalid capsule");
    let status = state
        .capsule
        .as_ref()
        .map(|capsule| capsule.status.to_string())
        .unwrap_or_else(|| "invalid".to_string());
    let line = Line::from(vec![
        Span::styled(
            title.to_string(),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(status, status_style(state)),
    ]);
    frame.render_widget(
        Paragraph::new(line)
            .block(Block::default().title(" Capsule ").borders(Borders::ALL))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_body(frame: &mut Frame<'_>, area: Rect, state: &WorkbenchState) {
    let [left, right] =
        Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)]).areas(area);
    frame.render_widget(
        List::new(overview_items(state))
            .block(Block::default().title(" Overview ").borders(Borders::ALL)),
        left,
    );
    frame.render_widget(render_active_panel(state), right);
}

fn render_footer(frame: &mut Frame<'_>, area: Rect, state: &WorkbenchState) {
    let message = if let Some(error) = &state.last_error {
        format!("refresh failed: {error}")
    } else {
        format!(
            "tab/right: next  shift-tab/left: previous  b: dashboard  r: refresh  q/esc/ctrl-c: quit  active: {}",
            state.active_panel.title()
        )
    };
    frame.render_widget(
        Paragraph::new(message)
            .block(Block::default().title(" Keys ").borders(Borders::ALL))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_active_panel(state: &WorkbenchState) -> Paragraph<'_> {
    let title = format!(" {} ", state.active_panel.title());
    match state.active_panel {
        Panel::Overview => Paragraph::new(overview_text(state))
            .block(Block::default().title(title).borders(Borders::ALL))
            .wrap(Wrap { trim: true }),
        Panel::Validation => Paragraph::new(validation_text(state))
            .block(Block::default().title(title).borders(Borders::ALL))
            .wrap(Wrap { trim: true }),
        Panel::Pr => Paragraph::new(pr_text(state))
            .block(Block::default().title(title).borders(Borders::ALL))
            .wrap(Wrap { trim: true }),
        Panel::Help => Paragraph::new(help_text())
            .block(Block::default().title(title).borders(Borders::ALL))
            .wrap(Wrap { trim: true }),
    }
}

fn overview_items(state: &WorkbenchState) -> Vec<ListItem<'static>> {
    let mut items = Vec::new();
    if let Some(capsule) = &state.capsule {
        items.push(ListItem::new(format!("id: {}", capsule.id)));
        items.push(ListItem::new(format!("branch: {}", capsule.branch)));
        items.push(ListItem::new(format!("base: {}", capsule.base_branch)));
        items.push(ListItem::new(format!(
            "issues: {}",
            join_numbers(&capsule.issues)
        )));
        items.push(ListItem::new(format!(
            "prs: {}",
            join_numbers(&capsule.pull_requests)
        )));
        items.push(ListItem::new(format!(
            "updated: {}",
            capsule.updated_at.to_rfc3339()
        )));
    } else {
        items.push(ListItem::new("invalid capsule"));
    }
    items.push(ListItem::new(format!(
        "validation: {}",
        if state.validation.valid {
            "valid"
        } else {
            "invalid"
        }
    )));
    items
}

fn overview_text(state: &WorkbenchState) -> String {
    match &state.capsule {
        Some(capsule) => format!(
            "{}\n\ncapsule: {}\n\nThis workbench reads codex-dev-core capsule JSON contracts and does not own policy logic.",
            capsule.objective, capsule.id
        ),
        None => format!(
            "Capsule failed validation for the supplied capsule path.\n\n{}",
            validation_text(state)
        ),
    }
}

fn validation_text(state: &WorkbenchState) -> String {
    if !state.validation.valid {
        return state
            .validation
            .errors
            .iter()
            .map(|error| redact_path_text(error, &state.capsule_path))
            .collect::<Vec<_>>()
            .join("\n");
    }

    let Some(verification) = &state.verification else {
        return "capsule is valid; no verification.json loaded".to_string();
    };

    let mut lines = vec![
        format!(
            "last checked: {}",
            verification.last_checked_at.to_rfc3339()
        ),
        "required gates:".to_string(),
    ];
    if verification.required.is_empty() {
        lines.push("- none recorded".to_string());
    } else {
        lines.extend(
            verification
                .required
                .iter()
                .map(|gate| format!("- {} [{}]", gate.name, gate.status)),
        );
    }
    if !verification.optional.is_empty() {
        lines.push("optional gates:".to_string());
        lines.extend(
            verification
                .optional
                .iter()
                .map(|gate| format!("- {} [{}]", gate.name, gate.status)),
        );
    }
    lines.join("\n")
}

fn pr_text(state: &WorkbenchState) -> String {
    let Some(pr) = &state.pr else {
        return "no PR evidence loaded".to_string();
    };

    let mut lines = vec![
        format!("target: {}", render_pr_label(pr)),
        format!("state: {}", pr.state),
        format!(
            "unresolved threads: {}",
            review_thread_unresolved_label(&pr.review_threads)
        ),
        format!(
            "threads checked: {}",
            pr.review_threads.last_checked_at.to_rfc3339()
        ),
    ];
    if let Some(url) = &pr.url {
        lines.push(format!("url: {url}"));
    }
    lines.push("checks:".to_string());
    if pr.checks.is_empty() {
        lines.push("- none recorded".to_string());
    } else {
        lines.extend(pr.checks.iter().map(render_check));
    }
    lines.join("\n")
}

fn render_check(check: &CheckRecord) -> String {
    let conclusion = check.conclusion.as_deref().unwrap_or("unknown");
    format!(
        "- {}: {} / {} at {}",
        check.name,
        check.status,
        conclusion,
        check.checked_at.to_rfc3339()
    )
}

fn help_text() -> &'static str {
    "Open a capsule with enter from dashboard or --capsule <dir>. Use --render-once for deterministic automation. The UI refreshes by rereading codex-dev-core JSON contract files."
}

fn status_style(state: &WorkbenchState) -> Style {
    if !state.validation.valid {
        return Style::default().fg(Color::Red);
    }

    match state.capsule.as_ref().map(|capsule| &capsule.status) {
        Some(CapsuleStatus::Merged | CapsuleStatus::Closed) => Style::default().fg(Color::Green),
        Some(CapsuleStatus::Blocked) => Style::default().fg(Color::Red),
        Some(CapsuleStatus::ReadyForPr | CapsuleStatus::InReview) => {
            Style::default().fg(Color::Yellow)
        }
        Some(CapsuleStatus::Active) | None => Style::default().fg(Color::Cyan),
    }
}

fn join_numbers(values: &[u64]) -> String {
    if values.is_empty() {
        return "none".to_string();
    }
    values
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Render a workbench state into a deterministic string buffer.
pub fn render_to_string(state: &WorkbenchState, width: u16, height: u16) -> Result<String> {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend)?;
    terminal.draw(|frame| render(frame, state))?;
    Ok(buffer_to_string(terminal.backend().buffer()))
}

/// Render a top-level app state into a deterministic string buffer.
pub fn render_app_to_string(state: &AppState, width: u16, height: u16) -> Result<String> {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend)?;
    terminal.draw(|frame| render_app(frame, state))?;
    Ok(buffer_to_string(terminal.backend().buffer()))
}

#[derive(Debug, PartialEq, Eq)]
/// Result of deterministic single-frame rendering.
pub struct RenderOnceResult {
    /// Rendered terminal buffer as plain text.
    pub output: String,
    /// Whether the rendered state should make CLI render-once exit successfully.
    pub valid: bool,
}

/// Load a capsule and render one deterministic frame without opening a terminal.
pub fn render_once(capsule_path: &Path, width: u16, height: u16) -> Result<RenderOnceResult> {
    ensure_render_dimensions(width, height)?;
    let state = WorkbenchState::load(capsule_path)?;
    let valid = state.validation.valid;
    let output = render_to_string(&state, width, height)?;
    Ok(RenderOnceResult { output, valid })
}

/// CLI-safe render-once wrapper that redacts capsule or dashboard-root paths from errors.
pub fn render_once_for_cli(
    capsule_path: Option<&Path>,
    dashboard_root: &Path,
    width: u16,
    height: u16,
) -> Result<RenderOnceResult> {
    ensure_render_dimensions(width, height)?;
    if let Some(capsule_path) = capsule_path {
        return render_once(capsule_path, width, height)
            .map_err(|error| sanitized_cli_error(error, capsule_path));
    }

    let state = AppState::load(None, dashboard_root)
        .map_err(|error| sanitized_path_error(error, dashboard_root, "<tasks-root>"))?;
    let output = redact_path_text_with_placeholder(
        &render_app_to_string(&state, width, height)?,
        dashboard_root,
        "<tasks-root>",
    );
    Ok(RenderOnceResult {
        output,
        valid: true,
    })
}

fn ensure_render_dimensions(width: u16, height: u16) -> Result<()> {
    if width == 0 || height == 0 {
        anyhow::bail!("--width and --height must be greater than 0");
    }
    Ok(())
}

fn buffer_to_string(buffer: &Buffer) -> String {
    let area = buffer.area;
    let mut lines = Vec::new();
    for y in area.y..area.y + area.height {
        let mut line = String::new();
        for x in area.x..area.x + area.width {
            if let Some(cell) = buffer.cell((x, y)) {
                line.push_str(cell.symbol());
            }
        }
        lines.push(line.trim_end().to_string());
    }
    let mut output = lines.join("\n");
    output.push('\n');
    output
}

fn redact_path_text(text: &str, path: &Path) -> String {
    redact_path_text_with_placeholder(text, path, "<capsule>")
}

fn redact_path_text_with_placeholder(text: &str, path: &Path, placeholder: &str) -> String {
    let path = path.display().to_string();
    if path.is_empty() {
        return text.to_string();
    }
    text.replace(&path, placeholder)
}

fn sanitized_cli_error(error: anyhow::Error, capsule_path: &Path) -> anyhow::Error {
    sanitized_path_error(error, capsule_path, "<capsule>")
}

fn sanitized_path_error(error: anyhow::Error, path: &Path, placeholder: &str) -> anyhow::Error {
    anyhow::anyhow!(
        "{}",
        redact_path_text_with_placeholder(&format!("{error:#}"), path, placeholder)
    )
}

fn interactive_tick_rate(tick_ms: u64) -> Result<Duration> {
    if tick_ms == 0 {
        anyhow::bail!("--tick-ms must be greater than 0");
    }
    Ok(Duration::from_millis(tick_ms))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    use chrono::{TimeZone, Utc};
    use codex_dev_core::{
        EvidenceSummary, GateRecord, InitArgs, POLICY_GATES_SCHEMA, PR_SCHEMA, PolicyGate,
        PolicyManifest, PolicyProfile, ReviewThreadSummary, VERIFICATION_SCHEMA, init_capsule,
    };
    use tempfile::tempdir;

    #[test]
    fn load_reads_codex_dev_core_contracts() {
        let temp = tempdir().expect("tempdir");
        let root = temp.path().join("tasks");
        let created_at = "2026-05-09T07:00:00Z".parse().expect("timestamp");
        let capsule = init_capsule(InitArgs {
            title: "TUI smoke".to_string(),
            objective: "TUI smoke".to_string(),
            branch: "feat/codex-dev-tui-workbench".to_string(),
            base_branch: "main".to_string(),
            issues: vec![28],
            pull_requests: Vec::new(),
            root,
            slug: None,
            id: Some("tui-smoke".to_string()),
            status: CapsuleStatus::Active,
            created_at,
            policy_manifest: PolicyManifest {
                schema: POLICY_GATES_SCHEMA.to_string(),
                profile: PolicyProfile::CodexDev,
                generated_at: created_at,
                gates: vec![fixture_policy_gate()],
            },
            force: false,
        })
        .expect("init capsule");

        let state = WorkbenchState::load(&capsule.path).expect("state");

        assert!(state.validation.valid);
        assert_eq!(state.capsule.as_ref().expect("capsule").issues, vec![28]);
        assert_eq!(state.pr.as_ref().expect("pr").state, "not_created");
    }

    #[test]
    fn panel_navigation_wraps() {
        let mut state = fixture_state();
        state.active_panel = Panel::Overview;

        state.next_panel();
        assert_eq!(state.active_panel, Panel::Validation);
        state.next_panel();
        assert_eq!(state.active_panel, Panel::Pr);
        state.previous_panel();
        assert_eq!(state.active_panel, Panel::Validation);
    }

    #[test]
    fn render_snapshot_contains_contract_summaries() {
        let state = fixture_state();

        let screen = render_to_string(&state, 100, 24).expect("render");

        assert!(screen.contains("codex-dev workbench"));
        assert!(screen.contains("TUI fixture"));
        assert!(screen.contains("validation: valid"));
        assert!(screen.contains("BjornMelin/dev-skills#28"));
    }

    #[test]
    fn render_once_reports_invalid_capsule_without_full_path() {
        let temp = tempdir().expect("tempdir");
        let capsule = temp.path().join("private-task-name");

        let result = render_once(&capsule, 100, 24).expect("render");

        assert!(!result.valid);
        assert!(result.output.contains("Capsule failed validation"));
        assert!(result.output.contains("<capsule>"));
        assert!(!result.output.contains(&capsule.display().to_string()));
    }

    #[test]
    fn render_once_dashboard_redacts_root_path() {
        let temp = tempdir().expect("tempdir");
        let root = temp.path().join("private-tasks-root");

        let result = render_once_for_cli(None, &root, 100, 24).expect("render");

        assert!(result.valid);
        assert!(result.output.contains("<tasks-root>"));
        assert!(!result.output.contains(&root.display().to_string()));
    }

    #[test]
    fn dashboard_discovers_and_renders_capsules() {
        let temp = tempdir().expect("tempdir");
        let root = temp.path().join("tasks");
        init_test_capsule(
            &root,
            "dashboard-one",
            "Dashboard one",
            CapsuleStatus::Active,
            0,
        );
        init_test_capsule(
            &root,
            "dashboard-two",
            "Dashboard two",
            CapsuleStatus::InReview,
            1,
        );

        let state = AppState::load(None, &root).expect("app state");
        let screen = render_app_to_string(&state, 120, 30).expect("render");

        assert!(state.capsule.is_none());
        assert_eq!(state.dashboard.capsules.len(), 2);
        assert!(screen.contains("codex-dev dashboard"));
        assert!(screen.contains("Dashboard one"));
        assert!(screen.contains("Dashboard two"));
        assert!(screen.contains("subagents:"));
        assert!(screen.contains("pr:"));
    }

    #[test]
    fn dashboard_surfaces_invalid_capsules_without_panics() {
        let temp = tempdir().expect("tempdir");
        let root = temp.path().join("tasks");
        fs::create_dir_all(root.join("broken-capsule")).expect("mkdir");

        let state = AppState::load(None, &root).expect("app state");
        let screen = render_app_to_string(&state, 120, 30).expect("render");

        assert_eq!(state.dashboard.capsules.len(), 1);
        assert!(!state.dashboard.capsules[0].validation.valid);
        assert!(screen.contains("broken-capsule"));
        assert!(screen.contains("invalid"));
        assert!(screen.contains("missing required file"));
    }

    #[test]
    fn dashboard_missing_root_renders_diagnostic() {
        let temp = tempdir().expect("tempdir");
        let root = temp.path().join("missing-tasks");

        let state = AppState::load(None, &root).expect("app state");
        let screen = render_app_to_string(&state, 120, 30).expect("render");

        assert!(state.dashboard.capsules.is_empty());
        assert_eq!(state.dashboard.diagnostics.len(), 1);
        assert!(screen.contains("dashboard root does not exist"));
        assert!(screen.contains("<tasks-root>"));
    }

    #[test]
    fn dashboard_navigation_filter_and_open_single_capsule() {
        let temp = tempdir().expect("tempdir");
        let root = temp.path().join("tasks");
        let active = init_test_capsule(
            &root,
            "dashboard-active",
            "Dashboard active",
            CapsuleStatus::Active,
            0,
        );
        init_test_capsule(
            &root,
            "dashboard-closed",
            "Dashboard closed",
            CapsuleStatus::Closed,
            1,
        );
        fs::create_dir_all(root.join("broken-capsule")).expect("mkdir");
        let mut state = AppState::load(None, &root).expect("app state");

        state.cycle_filter();
        assert_eq!(state.dashboard.filter, DashboardFilter::Active);
        assert_eq!(
            state
                .dashboard
                .selected_capsule()
                .map(|capsule| &capsule.path),
            Some(&active.path)
        );
        state.open_selected().expect("open selected");
        assert!(state.capsule.is_some());
        state.show_dashboard();
        assert!(state.capsule.is_none());
        state.cycle_sort();
        assert_eq!(state.dashboard.sort, DashboardSort::TitleAsc);
    }

    #[test]
    fn dashboard_refresh_preserves_filtered_selection() {
        let temp = tempdir().expect("tempdir");
        let root = temp.path().join("tasks");
        let _active_older = init_test_capsule(
            &root,
            "dashboard-active-older",
            "Dashboard active older",
            CapsuleStatus::Active,
            0,
        );
        let active_newer = init_test_capsule(
            &root,
            "dashboard-active-newer",
            "Dashboard active newer",
            CapsuleStatus::Active,
            1,
        );
        init_test_capsule(
            &root,
            "dashboard-closed-newest",
            "Dashboard closed newest",
            CapsuleStatus::Closed,
            2,
        );
        let mut state = AppState::load(None, &root).expect("app state");

        state.dashboard.filter = DashboardFilter::Active;
        state
            .dashboard
            .restore_selection(Some(active_newer.path.as_path()));
        assert_eq!(
            state
                .dashboard
                .selected_capsule()
                .map(|capsule| &capsule.path),
            Some(&active_newer.path)
        );

        state.dashboard.refresh();

        assert_eq!(
            state
                .dashboard
                .selected_capsule()
                .map(|capsule| &capsule.path),
            Some(&active_newer.path)
        );
    }

    #[test]
    fn dashboard_window_keeps_selected_capsule_visible() {
        let temp = tempdir().expect("tempdir");
        let root = temp.path().join("tasks");
        for index in 0..12 {
            init_test_capsule(
                &root,
                &format!("dashboard-{index:02}"),
                &format!("Dashboard {index:02}"),
                CapsuleStatus::Active,
                index,
            );
        }
        let mut state = AppState::load(None, &root).expect("app state");
        for _ in 0..11 {
            state.dashboard.next_item();
        }

        let screen = render_app_to_string(&state, 120, 12).expect("render");

        assert!(screen.contains("> Dashboard 00"));
    }

    #[test]
    fn non_authoritative_review_threads_are_not_reported_as_zero() {
        let state = fixture_state();
        let screen = render_to_string(&state, 100, 24).expect("render");
        let pr = state.pr.as_ref().expect("pr");

        assert_eq!(
            review_thread_unresolved_label(&pr.review_threads),
            "not checked"
        );
        assert!(screen.contains("unresolved threads: not checked"));
    }

    #[test]
    fn cli_error_sanitizer_redacts_capsule_path() {
        let temp = tempdir().expect("tempdir");
        let capsule = temp.path().join("private-task-name");

        let error = sanitized_cli_error(
            anyhow::anyhow!("failed to read {}", capsule.display()),
            &capsule,
        );

        let message = format!("{error:#}");
        assert!(message.contains("<capsule>"));
        assert!(!message.contains(&capsule.display().to_string()));
    }

    #[test]
    fn interactive_tick_rate_rejects_zero_delay() {
        let error = interactive_tick_rate(0).expect_err("zero tick rate fails");
        assert!(format!("{error:#}").contains("--tick-ms must be greater than 0"));
        assert_eq!(
            interactive_tick_rate(1).expect("positive tick"),
            Duration::from_millis(1)
        );
    }

    #[test]
    fn render_once_rejects_zero_dimensions() {
        let temp = tempdir().expect("tempdir");
        let root = temp.path().join("tasks");

        for (width, height) in [(0, 24), (100, 0)] {
            let error = render_once_for_cli(None, &root, width, height)
                .expect_err("zero render dimension fails");
            assert!(format!("{error:#}").contains("--width and --height must be greater than 0"));
        }
    }

    #[test]
    fn refresh_error_replaces_visible_contract_state() {
        let mut state = fixture_state();
        let message = "failed to parse <capsule>/verification.json".to_string();

        state.replace_contracts_with_error(message.clone());

        assert!(!state.validation.valid);
        assert_eq!(state.validation.errors, vec![message.clone()]);
        assert!(state.capsule.is_none());
        assert!(state.verification.is_none());
        assert!(state.pr.is_none());
        assert_eq!(state.last_error.as_deref(), Some(message.as_str()));
    }

    #[test]
    fn restore_guard_runs_cleanup_once() {
        let calls = Rc::new(RefCell::new(0usize));
        {
            let calls = Rc::clone(&calls);
            let mut guard = RestoreGuard::new(move || {
                *calls.borrow_mut() += 1;
            });
            guard.restore_now();
        }

        assert_eq!(*calls.borrow(), 1);
    }

    #[test]
    fn restore_guard_runs_cleanup_on_drop() {
        let calls = Rc::new(RefCell::new(0usize));
        {
            let calls = Rc::clone(&calls);
            let _guard = RestoreGuard::new(move || {
                *calls.borrow_mut() += 1;
            });
        }

        assert_eq!(*calls.borrow(), 1);
    }

    fn init_test_capsule(
        root: &Path,
        id: &str,
        title: &str,
        status: CapsuleStatus,
        offset_hours: i64,
    ) -> codex_dev_core::InitResult {
        let created_at = Utc
            .with_ymd_and_hms(2026, 5, 9, 7 + offset_hours as u32, 0, 0)
            .unwrap();
        init_capsule(InitArgs {
            title: title.to_string(),
            objective: format!("Objective for {title}"),
            branch: format!("feat/{id}"),
            base_branch: "main".to_string(),
            issues: vec![28],
            pull_requests: Vec::new(),
            root: root.to_path_buf(),
            slug: None,
            id: Some(id.to_string()),
            status,
            created_at,
            policy_manifest: PolicyManifest {
                schema: POLICY_GATES_SCHEMA.to_string(),
                profile: PolicyProfile::CodexDevTui,
                generated_at: created_at,
                gates: vec![fixture_policy_gate()],
            },
            force: false,
        })
        .expect("init capsule")
    }

    fn fixture_policy_gate() -> PolicyGate {
        PolicyGate {
            id: "docs-links".to_string(),
            name: "Docs links".to_string(),
            command: vec![
                "python3".to_string(),
                "tools/docs/check_links.py".to_string(),
                "docs".to_string(),
                "README.md".to_string(),
                "AGENTS.md".to_string(),
            ],
            source: "test-fixture".to_string(),
            working_directory: ".".to_string(),
            required_tools: vec!["python3".to_string()],
            required: true,
            network: false,
            secrets: false,
            failure_interpretation: "Fixture policy gate for TUI tests.".to_string(),
        }
    }

    fn fixture_state() -> WorkbenchState {
        let checked_at = Utc.with_ymd_and_hms(2026, 5, 9, 7, 0, 0).unwrap();
        WorkbenchState {
            capsule_path: PathBuf::from("/tmp/tui-fixture"),
            validation: ValidationResult {
                path: PathBuf::from("/tmp/tui-fixture"),
                valid: true,
                errors: Vec::new(),
            },
            capsule: Some(StatusResult {
                path: PathBuf::from("/tmp/tui-fixture"),
                id: "tui-fixture".to_string(),
                title: "TUI fixture".to_string(),
                status: CapsuleStatus::InReview,
                objective: "Render capsule state without owning policy logic.".to_string(),
                branch: "feat/codex-dev-tui-workbench".to_string(),
                base_branch: "main".to_string(),
                issues: vec![28],
                pull_requests: vec![35],
                updated_at: checked_at,
                evidence: EvidenceSummary {
                    total: 0,
                    by_kind: Vec::new(),
                },
            }),
            verification: Some(Verification {
                schema: VERIFICATION_SCHEMA.to_string(),
                required: vec![GateRecord {
                    name: "docs-links".to_string(),
                    command: "python3 tools/docs/check_links.py docs README.md AGENTS.md"
                        .to_string(),
                    status: "passed".to_string(),
                }],
                optional: Vec::new(),
                last_checked_at: checked_at,
            }),
            pr: Some(PrEvidence {
                schema: PR_SCHEMA.to_string(),
                repository: Some("BjornMelin/dev-skills".to_string()),
                number: Some(28),
                url: Some("https://github.com/BjornMelin/dev-skills/pull/28".to_string()),
                state: "open".to_string(),
                is_draft: Some(false),
                mergeable: Some("mergeable".to_string()),
                merge_state_status: Some("clean".to_string()),
                review_decision: Some("approved".to_string()),
                head_sha: Some("abc123".to_string()),
                head_ref_name: Some("feat/codex-dev-tui-workbench".to_string()),
                base_ref_name: Some("main".to_string()),
                base_ref_oid: Some("def456".to_string()),
                checks: vec![CheckRecord {
                    name: "CodeRabbit".to_string(),
                    status: "completed".to_string(),
                    conclusion: Some("success".to_string()),
                    url: None,
                    checked_at,
                }],
                review_threads: ReviewThreadSummary {
                    unresolved: 0,
                    total: 0,
                    resolved: 0,
                    outdated: 0,
                    authoritative: false,
                    last_checked_at: checked_at,
                },
                sources: Vec::new(),
            }),
            active_panel: Panel::Pr,
            last_error: None,
        }
    }
}
