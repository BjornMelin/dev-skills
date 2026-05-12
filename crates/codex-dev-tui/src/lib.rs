use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::Result;
use chrono::Utc;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use codex_dev_core::{
    CapsuleStatus, CheckRecord, EvidenceKind, EvidenceRecord, OrchestrationDiagnosticSeverity,
    OrchestrationRunReport, PR_AGENT_HOSTED_ACTION_SCHEMA, PR_AGENT_READINESS_SCHEMA,
    PR_AGENT_STATE_SCHEMA, PrAgentHostedActionReport, PrAgentHostedActionStatus,
    PrAgentReadinessActionStatus, PrAgentReadinessReport, PrAgentReadinessStatus, PrAgentSeverity,
    PrAgentStateReport, PrEvidence, ReviewThreadSummary, SkillInventoryArgs,
    SkillInventoryDiagnosticSeverity, SkillsInventoryReport, StatusResult, Subagents,
    TaskIndexEntry, TaskIndexReport, TaskRootStatus, ValidationResult, Verification,
    capsule_status, orchestration_run, read_json, render_command, render_pr_label,
    skills_inventory, task_index, validate_capsule,
};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::backend::{Backend, TestBackend};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};
use ratatui::{Frame, Terminal};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

pub const TUI_OPERATOR_PANELS_SCHEMA: &str = "tui_operator_panels.v1";
const ORCHESTRATION_STALE_AFTER_MINUTES: u64 = 120;

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
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generate shell completions for local installation.
    Completions {
        #[arg(value_enum)]
        shell: Shell,
    },
    /// Generate a roff manpage for local installation.
    Manpage,
}

/// Parse CLI arguments and run either the interactive TUI or deterministic render mode.
pub fn run() -> Result<()> {
    let Cli {
        capsule,
        root,
        render_once,
        width,
        height,
        tick_ms,
        command,
    } = Cli::parse();

    match command {
        Some(Commands::Completions { shell }) => {
            print!("{}", render_completion(shell)?);
            return Ok(());
        }
        Some(Commands::Manpage) => {
            print!("{}", render_manpage()?);
            return Ok(());
        }
        None => {}
    }

    if render_once {
        let result = render_once_for_cli(capsule.as_deref(), &root, width, height)?;
        print!("{}", result.output);
        if !result.valid {
            anyhow::bail!("invalid capsule; see render output for validation details");
        }
        return Ok(());
    }

    run_interactive(capsule.as_deref(), &root, interactive_tick_rate(tick_ms)?).map_err(|error| {
        match capsule.as_deref() {
            Some(capsule) => sanitized_cli_error(error, capsule),
            None => sanitized_path_error(error, &root, "<tasks-root>"),
        }
    })
}

fn render_completion(shell: Shell) -> Result<String> {
    let mut command = Cli::command();
    let binary_name = command.get_name().to_string();
    let mut buffer = Vec::new();
    clap_complete::generate(shell, &mut command, binary_name, &mut buffer);
    Ok(String::from_utf8(buffer)?)
}

fn render_manpage() -> Result<String> {
    let command = Cli::command();
    let mut buffer = Vec::new();
    clap_mangen::Man::new(command).render(&mut buffer)?;
    Ok(String::from_utf8(buffer)?)
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
    Evidence,
    Subagents,
    Orchestration,
    PrAgent,
    NextActions,
    Validation,
    Pr,
    Help,
}

impl Panel {
    fn next(self) -> Self {
        match self {
            Self::Overview => Self::Evidence,
            Self::Evidence => Self::Subagents,
            Self::Subagents => Self::Orchestration,
            Self::Orchestration => Self::Pr,
            Self::Pr => Self::PrAgent,
            Self::PrAgent => Self::NextActions,
            Self::NextActions => Self::Validation,
            Self::Validation => Self::Help,
            Self::Help => Self::Overview,
        }
    }

    fn previous(self) -> Self {
        match self {
            Self::Overview => Self::Help,
            Self::Evidence => Self::Overview,
            Self::Subagents => Self::Evidence,
            Self::Orchestration => Self::Subagents,
            Self::Pr => Self::Orchestration,
            Self::PrAgent => Self::Pr,
            Self::NextActions => Self::PrAgent,
            Self::Validation => Self::NextActions,
            Self::Help => Self::Validation,
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Evidence => "Evidence",
            Self::Subagents => "Subagents",
            Self::Orchestration => "Orchestration",
            Self::Validation => "Validation",
            Self::Pr => "PR",
            Self::PrAgent => "PR Agent",
            Self::NextActions => "Next Actions",
            Self::Help => "Help",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
/// Read-only composition contract for operator-focused TUI panels.
pub struct TuiOperatorPanelsReport {
    pub schema: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skill_inventory: Option<SkillsInventoryReport>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_index: Option<TaskIndexReport>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub orchestration: Vec<OrchestrationRunReport>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub next_actions: Vec<TuiNextActionExport>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
/// Command or operator action rendered by the TUI without executing it.
pub struct TuiNextActionExport {
    pub source: String,
    pub id: String,
    pub priority: String,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
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
    fn from_task_index_entry(task: &TaskIndexEntry) -> Self {
        let path = task.path.clone();
        let validation = ValidationResult {
            path: path.clone(),
            valid: task.valid,
            errors: task.errors.clone(),
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

        let verification = load_optional_contract(&path, "verification.json", || {
            read_json(&path.join("verification.json"))
        });
        let subagents = load_optional_contract(&path, "subagents.json", || {
            read_json(&path.join("subagents.json"))
        });
        let pr = load_optional_contract(&path, "pr.json", || read_json(&path.join("pr.json")));
        let mut diagnostics = Vec::new();
        diagnostics.extend(verification.1);
        diagnostics.extend(subagents.1);
        diagnostics.extend(pr.1);

        let capsule = task.capsule.clone();
        let display_title = capsule
            .as_ref()
            .map(|capsule| capsule.title.clone())
            .unwrap_or_else(|| fallback_dashboard_title(&path));
        let status_label = capsule
            .as_ref()
            .map(|capsule| capsule.status.to_string())
            .unwrap_or_else(|| "valid".to_string());
        let updated_label = capsule
            .as_ref()
            .map(|capsule| capsule.updated_at.to_rfc3339())
            .unwrap_or_else(|| "unknown".to_string());

        Self {
            path,
            display_title,
            status_label,
            updated_label,
            validation,
            capsule,
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
    pub operator: TuiOperatorPanelsReport,
    pub filtered_indices: Vec<usize>,
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
        let operator = dashboard_operator_report(&root);
        let mut diagnostics = operator
            .task_index
            .as_ref()
            .map(|report| report.diagnostics.clone())
            .unwrap_or_default();
        let mut capsules = Vec::new();
        if let Some(index) = &operator.task_index {
            capsules.extend(
                index
                    .tasks
                    .iter()
                    .map(DashboardCapsule::from_task_index_entry),
            );
        } else {
            diagnostics.push("task index report was not loaded".to_string());
        }
        sort_dashboard_capsules(&mut capsules, sort);
        let mut state = Self {
            root,
            capsules,
            operator,
            filtered_indices: Vec::new(),
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
        self.filtered_indices
            .get(self.selected)
            .and_then(|index| self.capsules.get(*index))
    }

    fn refresh_filtered_indices(&mut self) {
        self.filtered_indices = self
            .capsules
            .iter()
            .enumerate()
            .filter_map(|(index, capsule)| capsule.matches_filter(self.filter).then_some(index))
            .collect();
    }

    fn next_item(&mut self) {
        let len = self.filtered_indices.len();
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
        self.refresh_filtered_indices();
        self.selected = 0;
        self.clamp_selection();
    }

    fn cycle_sort(&mut self) {
        let selected_path = self.selected_capsule().map(|capsule| capsule.path.clone());
        self.sort = self.sort.next();
        sort_dashboard_capsules(&mut self.capsules, self.sort);
        self.refresh_filtered_indices();
        self.restore_selection(selected_path.as_deref());
    }

    fn restore_selection(&mut self, selected_path: Option<&Path>) {
        self.refresh_filtered_indices();
        self.selected = selected_path
            .and_then(|path| {
                self.filtered_indices
                    .iter()
                    .position(|index| self.capsules[*index].path == path)
            })
            .unwrap_or(0);
        self.clamp_selection();
    }

    fn clamp_selection(&mut self) {
        let len = self.filtered_indices.len();
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
    pub evidence: Vec<EvidenceRecord>,
    pub verification: Option<Verification>,
    pub subagents: Option<Subagents>,
    pub pr: Option<PrEvidence>,
    pub pr_agent_state: Option<PrAgentStateReport>,
    pub pr_readiness: Option<PrAgentReadinessReport>,
    pub pr_agent_actions: Vec<PrAgentHostedActionReport>,
    pub operator: TuiOperatorPanelsReport,
    pub diagnostics: Vec<String>,
    pub active_panel: Panel,
    pub last_error: Option<String>,
}

impl WorkbenchState {
    /// Validate and load the capsule contracts used by the TUI.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let validation = validate_capsule(&path)?;
        let (
            capsule,
            evidence,
            verification,
            subagents,
            pr,
            pr_agent_state,
            pr_readiness,
            pr_agent_actions,
            diagnostics,
        ) = if validation.valid {
            let mut diagnostics = Vec::new();
            let (capsule, capsule_diagnostics) =
                load_optional_contract(&path, "capsule", || capsule_status(&path));
            let (evidence, evidence_diagnostics) = load_evidence_records(&path);
            let (verification, verification_diagnostics) =
                load_optional_contract(&path, "verification.json", || {
                    read_json(&path.join("verification.json"))
                });
            let (subagents, subagent_diagnostics) =
                load_optional_contract(&path, "subagents.json", || {
                    read_json(&path.join("subagents.json"))
                });
            let (pr, pr_diagnostics) =
                load_optional_contract(&path, "pr.json", || read_json(&path.join("pr.json")));
            let (pr_agent_state, pr_agent_state_diagnostics) = load_optional_schema_contract(
                &path,
                "pr-agent-state.json",
                PR_AGENT_STATE_SCHEMA,
                |report: &PrAgentStateReport| &report.schema,
            );
            let (pr_readiness, pr_readiness_diagnostics) = load_optional_schema_contract(
                &path,
                "pr-readiness.json",
                PR_AGENT_READINESS_SCHEMA,
                |report: &PrAgentReadinessReport| &report.schema,
            );
            let (pr_agent_actions, action_diagnostics) = load_pr_agent_actions(&path);
            diagnostics.extend(capsule_diagnostics);
            diagnostics.extend(evidence_diagnostics);
            diagnostics.extend(verification_diagnostics);
            diagnostics.extend(subagent_diagnostics);
            diagnostics.extend(pr_diagnostics);
            diagnostics.extend(pr_agent_state_diagnostics);
            diagnostics.extend(pr_readiness_diagnostics);
            diagnostics.extend(action_diagnostics);
            (
                capsule,
                evidence,
                verification,
                subagents,
                pr,
                pr_agent_state,
                pr_readiness,
                pr_agent_actions,
                diagnostics,
            )
        } else {
            (
                None,
                Vec::new(),
                None,
                None,
                None,
                None,
                None,
                Vec::new(),
                Vec::new(),
            )
        };

        let mut operator = capsule_operator_report(
            &path,
            subagents.as_ref(),
            pr_agent_state.as_ref(),
            pr_readiness.as_ref(),
            &pr_agent_actions,
        );
        operator.diagnostics.extend(diagnostics.clone());

        Ok(Self {
            capsule_path: path,
            validation,
            capsule,
            evidence,
            verification,
            subagents,
            pr,
            pr_agent_state,
            pr_readiness,
            pr_agent_actions,
            operator,
            diagnostics,
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
        self.evidence.clear();
        self.verification = None;
        self.subagents = None;
        self.pr = None;
        self.pr_agent_state = None;
        self.pr_readiness = None;
        self.pr_agent_actions.clear();
        self.operator = TuiOperatorPanelsReport {
            schema: TUI_OPERATOR_PANELS_SCHEMA.to_string(),
            skill_inventory: None,
            task_index: None,
            orchestration: Vec::new(),
            next_actions: Vec::new(),
            diagnostics: vec![message.clone()],
        };
        self.diagnostics = vec![message.clone()];
        self.last_error = Some(message);
    }
}

fn optional_regular_file(
    file_path: &Path,
    capsule_path: &Path,
    label: &str,
) -> Result<bool, String> {
    let metadata = match file_path.symlink_metadata() {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => {
            return Err(format!(
                "{label}: {}",
                redact_path_text(
                    &format!("failed to inspect {}: {error}", file_path.display()),
                    capsule_path
                )
            ));
        }
    };
    if metadata.file_type().is_symlink() {
        return Err(format!("{label}: symlinks are not supported"));
    }
    if !metadata.file_type().is_file() {
        return Err(format!("{label}: expected a regular file"));
    }
    Ok(true)
}

fn required_regular_file(file_path: &Path, capsule_path: &Path, label: &str) -> Result<(), String> {
    match optional_regular_file(file_path, capsule_path, label)? {
        true => Ok(()),
        false => Err(format!("{label}: missing file")),
    }
}

fn load_optional_schema_contract<T>(
    path: &Path,
    file: &str,
    expected_schema: &'static str,
    schema: fn(&T) -> &str,
) -> (Option<T>, Vec<String>)
where
    T: DeserializeOwned,
{
    let file_path = path.join(file);
    match optional_regular_file(&file_path, path, file) {
        Ok(false) => return (None, Vec::new()),
        Ok(true) => {}
        Err(error) => return (None, vec![error]),
    }
    load_optional_contract(path, file, || {
        let value: T = read_json(&file_path)?;
        if schema(&value) != expected_schema {
            anyhow::bail!("{file} schema must be {expected_schema}");
        }
        Ok(value)
    })
}

fn load_evidence_records(path: &Path) -> (Vec<EvidenceRecord>, Vec<String>) {
    let evidence_path = path.join("evidence.jsonl");
    let file = match fs::File::open(&evidence_path) {
        Ok(file) => file,
        Err(error) => {
            return (
                Vec::new(),
                vec![format!(
                    "evidence.jsonl: {}",
                    redact_path_text(
                        &format!("failed to read {}: {error}", evidence_path.display()),
                        path
                    )
                )],
            );
        }
    };

    let mut records = Vec::new();
    let mut diagnostics = Vec::new();
    for (index, line) in BufReader::new(file).lines().enumerate() {
        let line = match line {
            Ok(line) => line,
            Err(error) => {
                diagnostics.push(format!("evidence.jsonl line {}: {error}", index + 1));
                continue;
            }
        };
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<EvidenceRecord>(&line) {
            Ok(record) => records.push(record),
            Err(error) => diagnostics.push(format!("evidence.jsonl line {}: {error}", index + 1)),
        }
    }
    (records, diagnostics)
}

fn load_pr_agent_actions(path: &Path) -> (Vec<PrAgentHostedActionReport>, Vec<String>) {
    let actions_root = path.join("pr-agent-actions");
    match actions_root.symlink_metadata() {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return (
                Vec::new(),
                vec!["pr-agent-actions: symlinks are not supported".to_string()],
            );
        }
        Ok(metadata) if !metadata.file_type().is_dir() => {
            return (
                Vec::new(),
                vec!["pr-agent-actions: expected a directory".to_string()],
            );
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return (Vec::new(), Vec::new());
        }
        Err(error) => {
            return (
                Vec::new(),
                vec![format!(
                    "pr-agent-actions: {}",
                    redact_path_text(
                        &format!("failed to inspect {}: {error}", actions_root.display()),
                        path
                    )
                )],
            );
        }
    }

    let entries = match fs::read_dir(&actions_root) {
        Ok(entries) => entries,
        Err(error) => {
            return (
                Vec::new(),
                vec![format!(
                    "pr-agent-actions: {}",
                    redact_path_text(
                        &format!("failed to read {}: {error}", actions_root.display()),
                        path
                    )
                )],
            );
        }
    };

    let mut reports = Vec::new();
    let mut diagnostics = Vec::new();
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                diagnostics.push(format!("pr-agent-actions entry: {error}"));
                continue;
            }
        };
        match entry.file_type() {
            Ok(file_type) if file_type.is_dir() => {}
            Ok(_) => continue,
            Err(error) => {
                diagnostics.push(format!(
                    "pr-agent-actions {}: {error}",
                    entry.file_name().to_string_lossy()
                ));
                continue;
            }
        }
        let plan_path = entry.path().join("plan.json");
        let label = format!(
            "pr-agent-actions/{}/plan.json",
            entry.file_name().to_string_lossy()
        );
        if let Err(error) = required_regular_file(&plan_path, path, &label) {
            diagnostics.push(error);
            continue;
        }
        match read_json::<PrAgentHostedActionReport>(&plan_path) {
            Ok(report) if report.schema == PR_AGENT_HOSTED_ACTION_SCHEMA => reports.push(report),
            Ok(_) => diagnostics.push(format!(
                "pr-agent-actions/{}/plan.json: plan.json schema must be {PR_AGENT_HOSTED_ACTION_SCHEMA}",
                entry.file_name().to_string_lossy()
            )),
            Err(error) => diagnostics.push(format!(
                "pr-agent-actions/{}/plan.json: {}",
                entry.file_name().to_string_lossy(),
                redact_path_text(&format!("{error:#}"), path)
            )),
        }
    }
    reports.sort_by(|left, right| {
        left.generated_at
            .cmp(&right.generated_at)
            .then_with(|| left.plan_id.cmp(&right.plan_id))
    });
    (reports, diagnostics)
}

fn dashboard_operator_report(root: &Path) -> TuiOperatorPanelsReport {
    let mut diagnostics = Vec::new();
    let repo_root = match dashboard_repo_root(root) {
        Ok(repo_root) => Some(repo_root),
        Err(error) => {
            diagnostics.push(format!("repo root: {error:#}"));
            None
        }
    };
    let skill_inventory = match repo_root {
        Some(repo_root) => match skills_inventory(SkillInventoryArgs {
            repo_root: Some(repo_root),
            checked_at: Some(Utc::now()),
        }) {
            Ok(report) => Some(report),
            Err(error) => {
                diagnostics.push(format!("skill inventory: {error:#}"));
                None
            }
        },
        None => None,
    };
    let task_index = match task_index(root) {
        Ok(report) => Some(report),
        Err(error) => {
            diagnostics.push(format!("task index: {error:#}"));
            None
        }
    };

    TuiOperatorPanelsReport {
        schema: TUI_OPERATOR_PANELS_SCHEMA.to_string(),
        skill_inventory,
        task_index,
        orchestration: Vec::new(),
        next_actions: Vec::new(),
        diagnostics,
    }
}

fn readiness_next_action_command(
    readiness: &PrAgentReadinessReport,
    action: &codex_dev_core::PrAgentReadinessAction,
) -> Option<String> {
    let pr = &readiness.attempts.last()?.pr;
    let repo = pr.repository.as_ref()?;
    let number = pr.number?;
    let mut command = vec![
        "codex-dev".to_string(),
        "pr".to_string(),
        "readiness".to_string(),
        "--capsule".to_string(),
        "CAPSULE_DIR".to_string(),
        "--repo".to_string(),
        repo.clone(),
        "--number".to_string(),
        number.to_string(),
    ];
    match action.kind.as_str() {
        "merge" => command.push("--merge".to_string()),
        "rerun_failed_jobs" => command.push("--rerun-failed".to_string()),
        _ => return None,
    }
    if action.status == PrAgentReadinessActionStatus::Planned {
        command.push("--apply".to_string());
    }
    Some(render_command(&command))
}

fn hosted_action_next_action_command(report: &PrAgentHostedActionReport) -> Option<String> {
    let mut command = vec![
        "codex-dev".to_string(),
        "pr".to_string(),
        "agent-action".to_string(),
        "--capsule".to_string(),
        "CAPSULE_DIR".to_string(),
        "--repo".to_string(),
        report.repository.clone(),
        "--number".to_string(),
        report.number.to_string(),
        "--plan-id".to_string(),
        report.plan_id.clone(),
        "--action".to_string(),
        report.action.kind.clone(),
    ];
    match report.action.kind.as_str() {
        "post-issue-comment" => {
            command.push("--body-file".to_string());
            command.push("BODY.md".to_string());
        }
        "reply-review-comment" => {
            command.push("--review-comment-id".to_string());
            command.push(hosted_action_target_value(
                &report.action.target,
                "review-comment:",
                "COMMENT_ID",
            ));
            command.push("--body-file".to_string());
            command.push("BODY.md".to_string());
        }
        "resolve-review-thread" | "unresolve-review-thread" => {
            command.push("--thread-id".to_string());
            command.push(hosted_action_target_value(
                &report.action.target,
                "review-thread:",
                "THREAD_ID",
            ));
        }
        "add-labels" | "remove-labels" => {
            let labels = hosted_action_target_value(&report.action.target, "labels:", "LABEL");
            for label in labels.split(',').filter(|label| !label.is_empty()) {
                command.push("--label".to_string());
                command.push(label.to_string());
            }
        }
        "rerun-failed-jobs" => {
            command.push("--run-id".to_string());
            command.push(hosted_action_target_value(
                &report.action.target,
                "workflow-run:",
                "RUN_ID",
            ));
        }
        _ => return None,
    }
    if report.action.requires_apply && !report.apply_requested {
        command.push("--apply".to_string());
    }
    Some(render_command(&command))
}

fn hosted_action_target_value(target: &str, prefix: &str, fallback: &str) -> String {
    target
        .strip_prefix(prefix)
        .filter(|value| !value.is_empty())
        .or_else(|| (!target.is_empty() && !target.contains(':')).then_some(target))
        .unwrap_or(fallback)
        .to_string()
}

fn dashboard_repo_root(root: &Path) -> Result<PathBuf> {
    let absolute_root = if root.is_absolute() {
        root.to_path_buf()
    } else {
        std::env::current_dir()?.join(root)
    };
    let existing = nearest_existing_ancestor(&absolute_root).ok_or_else(|| {
        anyhow::anyhow!("failed to find an existing ancestor for the dashboard task root")
    })?;
    let canonical = fs::canonicalize(existing)?;
    let start = if canonical.is_file() {
        canonical
            .parent()
            .ok_or_else(|| {
                anyhow::anyhow!("dashboard task root resolved to a file without parent")
            })?
            .to_path_buf()
    } else {
        canonical
    };
    start
        .ancestors()
        .find(|path| {
            path.join("Cargo.toml").is_file() && path.join("docs/runbooks/validation.md").is_file()
        })
        .map(Path::to_path_buf)
        .ok_or_else(|| {
            anyhow::anyhow!("failed to discover repository root from dashboard task root")
        })
}

fn nearest_existing_ancestor(path: &Path) -> Option<&Path> {
    path.ancestors().find(|ancestor| ancestor.exists())
}

fn capsule_operator_report(
    capsule_path: &Path,
    subagents: Option<&Subagents>,
    pr_agent_state: Option<&PrAgentStateReport>,
    pr_readiness: Option<&PrAgentReadinessReport>,
    pr_agent_actions: &[PrAgentHostedActionReport],
) -> TuiOperatorPanelsReport {
    let mut diagnostics = Vec::new();
    let mut orchestration = Vec::new();
    if let Some(subagents) = subagents {
        for batch in &subagents.batches {
            match orchestration_run(
                capsule_path,
                &batch.id,
                Utc::now(),
                ORCHESTRATION_STALE_AFTER_MINUTES,
            ) {
                Ok(report) => orchestration.push(report),
                Err(error) => diagnostics.push(format!(
                    "orchestration batch {}: {}",
                    batch.id,
                    redact_path_text(&format!("{error:#}"), capsule_path)
                )),
            }
        }
    }

    let mut next_actions = Vec::new();
    if let Some(report) = pr_agent_state {
        next_actions.extend(report.actions.iter().map(|action| TuiNextActionExport {
            source: "pr-agent-state".to_string(),
            id: action.id.clone(),
            priority: pr_agent_action_priority_label(action.priority).to_string(),
            summary: action.summary.clone(),
            command: None,
        }));
    }
    if let Some(readiness) = pr_readiness {
        next_actions.extend(readiness.actions.iter().map(|action| TuiNextActionExport {
            source: "pr-readiness".to_string(),
            id: action.id.clone(),
            priority: readiness_action_status_label(action.status).to_string(),
            summary: action.reason.clone(),
            command: readiness_next_action_command(readiness, action),
        }));
    }
    next_actions.extend(pr_agent_actions.iter().map(|report| {
        TuiNextActionExport {
            source: "pr-agent-hosted-action".to_string(),
            id: report.action.id.clone(),
            priority: if report.action.requires_apply {
                "requires-apply"
            } else {
                "info"
            }
            .to_string(),
            summary: report.action.summary.clone(),
            command: hosted_action_next_action_command(report),
        }
    }));
    for report in &orchestration {
        next_actions.extend(orchestration_next_actions(report));
    }
    next_actions.sort_by(|left, right| {
        left.source
            .cmp(&right.source)
            .then_with(|| left.id.cmp(&right.id))
    });

    TuiOperatorPanelsReport {
        schema: TUI_OPERATOR_PANELS_SCHEMA.to_string(),
        skill_inventory: None,
        task_index: None,
        orchestration,
        next_actions,
        diagnostics,
    }
}

fn orchestration_next_actions(report: &OrchestrationRunReport) -> Vec<TuiNextActionExport> {
    let mut actions = Vec::new();
    for diagnostic in &report.diagnostics {
        if diagnostic.severity != OrchestrationDiagnosticSeverity::Error {
            continue;
        }
        let Some(role) = diagnostic.role.as_deref() else {
            if diagnostic.code == "missing_synthesis" || diagnostic.code == "incomplete_synthesis" {
                actions.push(TuiNextActionExport {
                    source: "orchestration".to_string(),
                    id: format!("{}:{}", report.batch_id, diagnostic.code),
                    priority: "required".to_string(),
                    summary: diagnostic.message.clone(),
                    command: Some(orchestration_close_command(&report.batch_id)),
                });
            }
            continue;
        };
        actions.push(TuiNextActionExport {
            source: "orchestration".to_string(),
            id: format!("{}:{}:{role}", report.batch_id, diagnostic.code),
            priority: "required".to_string(),
            summary: diagnostic.message.clone(),
            command: Some(orchestration_record_command(&report.batch_id, role)),
        });
    }
    actions
}

fn orchestration_close_command(batch_id: &str) -> String {
    render_command(&[
        "codex-dev".to_string(),
        "--json".to_string(),
        "orchestration".to_string(),
        "close".to_string(),
        "--capsule".to_string(),
        "CAPSULE_DIR".to_string(),
        "--batch-id".to_string(),
        batch_id.to_string(),
        "--status".to_string(),
        "completed".to_string(),
        "--summary".to_string(),
        "SUMMARY".to_string(),
        "--human-verified".to_string(),
    ])
}

fn orchestration_record_command(batch_id: &str, role: &str) -> String {
    render_command(&[
        "codex-dev".to_string(),
        "--json".to_string(),
        "orchestration".to_string(),
        "record".to_string(),
        "--capsule".to_string(),
        "CAPSULE_DIR".to_string(),
        "--batch-id".to_string(),
        batch_id.to_string(),
        "--role".to_string(),
        role.to_string(),
        "--status".to_string(),
        "completed".to_string(),
        "--summary".to_string(),
        "SUMMARY".to_string(),
        "--disposition".to_string(),
        "accepted".to_string(),
        "--human-verified".to_string(),
    ])
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

#[cfg(all(test, unix))]
fn symlink_path(target: &Path, link: &Path) {
    std::os::unix::fs::symlink(target, link).expect("symlink");
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
    let visible = state.filtered_indices.len();
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
    let indices = &state.filtered_indices;
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
    append_dashboard_operator_panels(&mut lines, state);
    lines.push(String::new());

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

fn append_dashboard_operator_panels(lines: &mut Vec<String>, state: &DashboardState) {
    lines.push(format!("operator schema: {}", state.operator.schema));
    if let Some(index) = &state.operator.task_index {
        lines.push(format!(
            "task index: {} valid, {} invalid, {} total ({})",
            index.valid,
            index.invalid,
            index.total,
            task_root_status_label(index.root_status)
        ));
    } else {
        lines.push("task index: not loaded".to_string());
    }

    if let Some(inventory) = &state.operator.skill_inventory {
        lines.push(format!(
            "skill health: {} valid, {} invalid, {} total, diagnostics {}",
            inventory.valid,
            inventory.invalid,
            inventory.total,
            inventory.diagnostics.len()
        ));
        let underbuilt = inventory
            .skills
            .iter()
            .filter(|skill| !skill.underbuilt_signals.is_empty())
            .count();
        lines.push(format!("underbuilt signals: {underbuilt} skill(s)"));
        if let Some(diagnostic) = inventory.diagnostics.first() {
            lines.push(format!(
                "first skill diagnostic: {} [{}]",
                diagnostic.code,
                skill_inventory_severity_label(diagnostic.severity)
            ));
        }
    } else {
        lines.push("skill health: not loaded".to_string());
    }
    if !state.operator.diagnostics.is_empty() {
        lines.push("operator diagnostics:".to_string());
        lines.extend(state.operator.diagnostics.iter().map(|diagnostic| {
            format!(
                "- {}",
                redact_path_text_with_placeholder(diagnostic, &state.root, "<tasks-root>")
            )
        }));
    }
}

fn task_root_status_label(status: TaskRootStatus) -> &'static str {
    match status {
        TaskRootStatus::Ready => "ready",
        TaskRootStatus::Missing => "missing",
        TaskRootStatus::Unusable => "unusable",
    }
}

fn skill_inventory_severity_label(severity: SkillInventoryDiagnosticSeverity) -> &'static str {
    match severity {
        SkillInventoryDiagnosticSeverity::Info => "info",
        SkillInventoryDiagnosticSeverity::Warning => "warning",
        SkillInventoryDiagnosticSeverity::Error => "error",
    }
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
        Panel::Evidence => Paragraph::new(evidence_text(state))
            .block(Block::default().title(title).borders(Borders::ALL))
            .wrap(Wrap { trim: true }),
        Panel::Subagents => Paragraph::new(subagents_text(state))
            .block(Block::default().title(title).borders(Borders::ALL))
            .wrap(Wrap { trim: true }),
        Panel::Orchestration => Paragraph::new(orchestration_text(state))
            .block(Block::default().title(title).borders(Borders::ALL))
            .wrap(Wrap { trim: true }),
        Panel::Validation => Paragraph::new(validation_text(state))
            .block(Block::default().title(title).borders(Borders::ALL))
            .wrap(Wrap { trim: true }),
        Panel::Pr => Paragraph::new(pr_text(state))
            .block(Block::default().title(title).borders(Borders::ALL))
            .wrap(Wrap { trim: true }),
        Panel::PrAgent => Paragraph::new(pr_agent_text(state))
            .block(Block::default().title(title).borders(Borders::ALL))
            .wrap(Wrap { trim: true }),
        Panel::NextActions => Paragraph::new(next_actions_text(state))
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
    items.push(ListItem::new(format!(
        "evidence: {} record(s)",
        state.evidence.len()
    )));
    items.push(ListItem::new(format!(
        "subagents: {}",
        state
            .subagents
            .as_ref()
            .map(subagents_status_label)
            .unwrap_or_else(|| "not loaded".to_string())
    )));
    items.push(ListItem::new(format!(
        "pr agent: {}",
        pr_agent_status_label(state)
    )));
    items.push(ListItem::new(format!(
        "orchestration: {} run(s)",
        state.operator.orchestration.len()
    )));
    items.push(ListItem::new(format!(
        "next actions: {}",
        state.operator.next_actions.len()
    )));
    items
}

fn overview_text(state: &WorkbenchState) -> String {
    match &state.capsule {
        Some(capsule) => {
            let mut lines = vec![
                capsule.objective.clone(),
                String::new(),
                format!("capsule: {}", capsule.id),
                format!("evidence: {} loaded record(s)", state.evidence.len()),
                format!(
                    "subagents: {}",
                    state
                        .subagents
                        .as_ref()
                        .map(subagents_status_label)
                        .unwrap_or_else(|| "not loaded".to_string())
                ),
                format!("pr agent: {}", pr_agent_status_label(state)),
                format!("orchestration runs: {}", state.operator.orchestration.len()),
                format!("next actions: {}", state.operator.next_actions.len()),
                String::new(),
                "This workbench reads codex-dev-core capsule JSON contracts and does not own policy logic.".to_string(),
            ];
            append_artifact_diagnostics(&mut lines, state);
            lines.join("\n")
        }
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
    append_artifact_diagnostics(&mut lines, state);
    lines.join("\n")
}

fn evidence_text(state: &WorkbenchState) -> String {
    let mut lines = vec![format!("loaded records: {}", state.evidence.len())];
    if let Some(capsule) = &state.capsule {
        lines.push(format!(
            "capsule summary: {} total; {}",
            capsule.evidence.total,
            evidence_summary_text(&capsule.evidence.by_kind)
        ));
    }

    let warnings = evidence_warnings(state);
    if !warnings.is_empty() {
        lines.push("warnings:".to_string());
        lines.extend(warnings.into_iter().map(|warning| format!("- {warning}")));
    }

    if state.evidence.is_empty() {
        lines.push("no evidence records loaded".to_string());
        append_artifact_diagnostics(&mut lines, state);
        return lines.join("\n");
    }

    lines.push("by kind:".to_string());
    for (kind, count) in evidence_kind_counts(&state.evidence) {
        lines.push(format!("- {}: {count}", evidence_kind_label(kind)));
    }

    lines.push("recent records:".to_string());
    for record in state.evidence.iter().rev().take(6) {
        lines.push(format!(
            "- {} {}: {}",
            record.at.to_rfc3339(),
            evidence_kind_label(record.kind),
            record.summary
        ));
        if !record.source_ids.is_empty() {
            lines.push(format!("  sources: {}", record.source_ids.join(", ")));
        }
        if !record.artifacts.is_empty() {
            lines.push(format!(
                "  artifacts: {}",
                sanitize_list(&record.artifacts, &state.capsule_path)
            ));
        }
        if let Some(confidence) = record.confidence {
            lines.push(format!("  confidence: {confidence}/100"));
        }
        if let Some(residual_risk) = &record.residual_risk {
            lines.push(format!("  residual risk: {residual_risk}"));
        }
    }
    append_artifact_diagnostics(&mut lines, state);
    lines.join("\n")
}

fn subagents_text(state: &WorkbenchState) -> String {
    let Some(subagents) = &state.subagents else {
        let mut lines = vec!["no subagent evidence loaded".to_string()];
        append_artifact_diagnostics(&mut lines, state);
        return lines.join("\n");
    };

    let mut lines = vec![
        format!("schema: {}", subagents.schema),
        format!("batches: {}", subagents.batches.len()),
    ];
    if subagents.batches.is_empty() {
        lines.push("no subagent batches recorded".to_string());
        append_artifact_diagnostics(&mut lines, state);
        return lines.join("\n");
    }

    for batch in subagents.batches.iter().rev().take(5) {
        lines.push(format!("- batch {} [{}]", batch.id, batch.status));
        if let Some(task) = &batch.task {
            lines.push(format!("  task: {task}"));
        }
        if let Some(mode) = &batch.mode {
            lines.push(format!("  mode: {mode}"));
        }
        if let Some(scope) = &batch.scope {
            lines.push(format!("  scope: {scope}"));
        }
        let completed = batch
            .agents
            .iter()
            .filter(|agent| agent.status == "completed")
            .count();
        let verified = batch
            .agents
            .iter()
            .filter(|agent| agent.human_verified)
            .count();
        lines.push(format!(
            "  agents: {completed}/{} completed; {verified} human-verified",
            batch.agents.len()
        ));
        if !batch.registry_issues.is_empty() {
            lines.push(format!(
                "  registry issues: {}",
                sanitize_list(&batch.registry_issues, &state.capsule_path)
            ));
        }
        for agent in batch.agents.iter().take(4) {
            let disposition = agent.disposition.as_deref().unwrap_or("unclassified");
            lines.push(format!(
                "  - {} [{}; {disposition}]: {}",
                agent.role, agent.status, agent.summary
            ));
            if !agent.source_ids.is_empty() {
                lines.push(format!("    sources: {}", agent.source_ids.join(", ")));
            }
            if !agent.artifacts.is_empty() {
                lines.push(format!(
                    "    artifacts: {}",
                    sanitize_list(&agent.artifacts, &state.capsule_path)
                ));
            }
        }
        if let Some(synthesis) = &batch.synthesis {
            lines.push(format!(
                "  synthesis [{}; human_verified={}]: {}",
                synthesis.status, synthesis.human_verified, synthesis.summary
            ));
            if !synthesis.source_ids.is_empty() {
                lines.push(format!("    sources: {}", synthesis.source_ids.join(", ")));
            }
        }
    }
    append_artifact_diagnostics(&mut lines, state);
    lines.join("\n")
}

fn orchestration_text(state: &WorkbenchState) -> String {
    let mut lines = vec![format!("schema: {}", state.operator.schema)];
    if state.operator.orchestration.is_empty() {
        lines.push("no orchestration run reports loaded".to_string());
        append_operator_diagnostics(&mut lines, &state.operator, &state.capsule_path);
        return lines.join("\n");
    }

    for report in state.operator.orchestration.iter().rev().take(5) {
        lines.push(format!(
            "- batch {} [{}] complete={} roles {}/{} synthesis={}",
            report.batch_id,
            report.status,
            report.completion.complete,
            report.completion.human_verified,
            report.completion.expected,
            report.completion.synthesis_completed
        ));
        if !report.completion.missing.is_empty() {
            lines.push(format!(
                "  missing: {}",
                report.completion.missing.join(", ")
            ));
        }
        if !report.completion.extra.is_empty() {
            lines.push(format!(
                "  unexpected: {}",
                report.completion.extra.join(", ")
            ));
        }
        for agent in report.agents.iter().take(5) {
            let wait = agent.wait_status.as_deref().unwrap_or("not-recorded");
            let agent_id = agent.agent_id.as_deref().unwrap_or("missing-id");
            lines.push(format!(
                "  - {} [{}; wait={wait}; id={agent_id}; human_verified={}]",
                agent.role, agent.status, agent.human_verified
            ));
        }
        if report.diagnostics.is_empty() {
            lines.push("  diagnostics: none".to_string());
        } else {
            lines.push("  diagnostics:".to_string());
            for diagnostic in report.diagnostics.iter().take(6) {
                lines.push(format!(
                    "  - {} [{}]: {}",
                    diagnostic.code,
                    orchestration_severity_label(diagnostic.severity),
                    redact_path_text(&diagnostic.message, &state.capsule_path)
                ));
            }
        }
    }
    append_operator_diagnostics(&mut lines, &state.operator, &state.capsule_path);
    lines.join("\n")
}

fn next_actions_text(state: &WorkbenchState) -> String {
    let mut lines = vec![format!("schema: {}", state.operator.schema)];
    if state.operator.next_actions.is_empty() {
        lines.push("no next actions exported".to_string());
        append_operator_diagnostics(&mut lines, &state.operator, &state.capsule_path);
        return lines.join("\n");
    }

    for action in state.operator.next_actions.iter().take(10) {
        lines.push(format!(
            "- {}:{} [{}]: {}",
            action.source, action.id, action.priority, action.summary
        ));
        if let Some(command) = &action.command {
            lines.push(format!(
                "  command: {}",
                redact_path_text(command, &state.capsule_path)
            ));
        }
    }
    append_operator_diagnostics(&mut lines, &state.operator, &state.capsule_path);
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

fn pr_agent_text(state: &WorkbenchState) -> String {
    let mut lines = Vec::new();
    if let Some(pr) = &state.pr {
        lines.push(format!("pr snapshot: {}", render_pr_label(pr)));
        lines.push(format!("state: {}", pr.state));
        lines.push(format!(
            "review threads: {} unresolved of {} total ({})",
            review_thread_unresolved_label(&pr.review_threads),
            pr.review_threads.total,
            if pr.review_threads.authoritative {
                "authoritative"
            } else {
                "not authoritative"
            }
        ));
    }

    match &state.pr_agent_state {
        Some(report) => append_pr_agent_state(&mut lines, report),
        None => lines.push("state report: not loaded".to_string()),
    }
    match &state.pr_readiness {
        Some(readiness) => append_pr_readiness(&mut lines, readiness),
        None => lines.push("readiness report: not loaded".to_string()),
    }
    append_pr_agent_actions(&mut lines, &state.pr_agent_actions);
    append_artifact_diagnostics(&mut lines, state);
    lines.join("\n")
}

fn append_pr_agent_state(lines: &mut Vec<String>, report: &PrAgentStateReport) {
    lines.push(format!(
        "state report: {}#{} at {} ({})",
        report.repository,
        report.number,
        report.checked_at.to_rfc3339(),
        if report.dry_run { "dry-run" } else { "live" }
    ));
    let captured = report
        .sources
        .iter()
        .filter(|source| matches!(source.status, codex_dev_core::PrAgentSourceStatus::Captured))
        .count();
    let failed = report
        .sources
        .iter()
        .filter(|source| matches!(source.status, codex_dev_core::PrAgentSourceStatus::Failed))
        .count();
    lines.push(format!(
        "  sources: {captured} captured, {failed} failed, {} total",
        report.sources.len()
    ));
    if !report.sources.is_empty() {
        lines.push(format!(
            "  source ids: {}",
            report
                .sources
                .iter()
                .map(|source| source.id.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    let diagnostics = diagnostic_counts(&report.diagnostics);
    lines.push(format!("  diagnostics: {diagnostics}"));
    if report.actions.is_empty() {
        lines.push("  recommended actions: none".to_string());
    } else {
        lines.push("  recommended actions:".to_string());
        lines.extend(report.actions.iter().take(5).map(|action| {
            format!(
                "  - {} [{}]: {}",
                action.id,
                pr_agent_action_priority_label(action.priority),
                action.summary
            )
        }));
    }
}

fn append_pr_readiness(lines: &mut Vec<String>, readiness: &PrAgentReadinessReport) {
    lines.push(format!(
        "readiness: {} (ready={}; attempts={}; generated={})",
        readiness_status_label(readiness.final_status),
        readiness.ready,
        readiness.attempts.len(),
        readiness.generated_at.to_rfc3339()
    ));
    lines.push(format!(
        "  requested: apply={} rerun_failed={} merge={}",
        readiness.apply_requested, readiness.rerun_failed_requested, readiness.merge_requested
    ));
    if let Some(attempt) = readiness.attempts.last() {
        lines.push(format!(
            "  latest attempt {}: {} at {}",
            attempt.attempt,
            readiness_status_label(attempt.status),
            attempt.checked_at.to_rfc3339()
        ));
        lines.push(format!(
            "  comments: {} active, {} outdated",
            attempt.active_review_comments, attempt.outdated_review_comments
        ));
        if !attempt.blockers.is_empty() {
            lines.push(format!("  blockers: {}", attempt.blockers.join("; ")));
        }
        if !attempt.wait_reasons.is_empty() {
            lines.push(format!("  wait: {}", attempt.wait_reasons.join("; ")));
        }
        if !attempt.warnings.is_empty() {
            lines.push(format!("  warnings: {}", attempt.warnings.join("; ")));
        }
        if !attempt.failing_checks.is_empty() {
            lines.push(format!(
                "  failing checks: {}",
                attempt
                    .failing_checks
                    .iter()
                    .map(readiness_check_label)
                    .collect::<Vec<_>>()
                    .join("; ")
            ));
        }
        if !attempt.pending_checks.is_empty() {
            lines.push(format!(
                "  pending checks: {}",
                attempt
                    .pending_checks
                    .iter()
                    .map(readiness_check_label)
                    .collect::<Vec<_>>()
                    .join("; ")
            ));
        }
        let diagnostics = diagnostic_counts(&attempt.diagnostics);
        lines.push(format!("  attempt diagnostics: {diagnostics}"));
    }
    if readiness.actions.is_empty() {
        lines.push("  readiness actions: none".to_string());
    } else {
        lines.push("  readiness actions:".to_string());
        for action in readiness.actions.iter().take(5) {
            lines.push(format!(
                "  - {} {} [{}]: {}",
                action.id,
                action.kind,
                readiness_action_status_label(action.status),
                action.reason
            ));
        }
    }
}

fn append_pr_agent_actions(lines: &mut Vec<String>, actions: &[PrAgentHostedActionReport]) {
    if actions.is_empty() {
        lines.push("hosted action plans: none".to_string());
        return;
    }
    lines.push(format!("hosted action plans: {}", actions.len()));
    for report in actions.iter().rev().take(5) {
        let mode = hosted_action_mode(report);
        lines.push(format!(
            "- {} {} -> {} [{}]: {}",
            report.plan_id, report.action.kind, report.action.target, mode, report.action.summary
        ));
        lines.push(format!(
            "  apply_requested={} dry_run={} generated={}",
            report.apply_requested,
            report.dry_run,
            report.generated_at.to_rfc3339()
        ));
        if !report.diagnostics.is_empty() {
            lines.push(format!(
                "  diagnostics: {}",
                diagnostic_counts(&report.diagnostics)
            ));
        }
    }
}

fn evidence_summary_text(summary: &[codex_dev_core::EvidenceKindSummary]) -> String {
    if summary.is_empty() {
        return "no kind summary".to_string();
    }
    summary
        .iter()
        .map(|item| format!("{}={}", evidence_kind_label(item.kind), item.count))
        .collect::<Vec<_>>()
        .join(", ")
}

fn evidence_warnings(state: &WorkbenchState) -> Vec<String> {
    let mut warnings = Vec::new();
    if state.evidence.is_empty() {
        warnings.push("missing evidence records".to_string());
        return warnings;
    }
    if let Some(capsule) = &state.capsule
        && capsule.evidence.total != state.evidence.len() as u64
    {
        warnings.push(format!(
            "stale summary: capsule reports {} evidence record(s), loaded {}",
            capsule.evidence.total,
            state.evidence.len()
        ));
    }
    let missing_source_context = state
        .evidence
        .iter()
        .filter(|record| {
            matches!(
                record.kind,
                EvidenceKind::Decision
                    | EvidenceKind::Research
                    | EvidenceKind::Review
                    | EvidenceKind::Subagent
            ) && record.source_ids.is_empty()
        })
        .count();
    if missing_source_context > 0 {
        warnings.push(format!(
            "{missing_source_context} decision/research/review/subagent record(s) missing source IDs"
        ));
    }
    if !state
        .evidence
        .iter()
        .any(|record| record.kind == EvidenceKind::Decision)
    {
        warnings.push("no decision evidence recorded".to_string());
    }
    if !state
        .evidence
        .iter()
        .any(|record| record.kind == EvidenceKind::Research)
    {
        warnings.push("no research evidence recorded".to_string());
    }
    warnings
}

fn evidence_kind_counts(records: &[EvidenceRecord]) -> BTreeMap<EvidenceKind, usize> {
    let mut counts = BTreeMap::new();
    for record in records {
        *counts.entry(record.kind).or_default() += 1;
    }
    counts
}

fn evidence_kind_label(kind: EvidenceKind) -> &'static str {
    match kind {
        EvidenceKind::Command => "command",
        EvidenceKind::Subagent => "subagent",
        EvidenceKind::Review => "review",
        EvidenceKind::Ci => "ci",
        EvidenceKind::Decision => "decision",
        EvidenceKind::Research => "research",
        EvidenceKind::Manual => "manual",
        EvidenceKind::Output => "output",
    }
}

fn subagents_status_label(subagents: &Subagents) -> String {
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

fn pr_agent_status_label(state: &WorkbenchState) -> String {
    let mut parts = Vec::new();
    if let Some(report) = &state.pr_agent_state {
        parts.push(if report.dry_run {
            "state dry-run"
        } else {
            "state live"
        });
    }
    if let Some(readiness) = &state.pr_readiness {
        parts.push(readiness_status_label(readiness.final_status));
    }
    if !state.pr_agent_actions.is_empty() {
        parts.push("hosted actions");
    }
    if parts.is_empty() {
        "not loaded".to_string()
    } else {
        parts.join(", ")
    }
}

fn hosted_action_mode(report: &PrAgentHostedActionReport) -> String {
    match &report.execution {
        Some(execution) => hosted_action_status_label(execution.status).to_string(),
        None if report.apply_requested => "apply requested; not executed".to_string(),
        None => "dry-run plan".to_string(),
    }
}

fn hosted_action_status_label(status: PrAgentHostedActionStatus) -> &'static str {
    match status {
        PrAgentHostedActionStatus::Applied => "applied",
        PrAgentHostedActionStatus::SkippedDuplicate => "skipped duplicate",
        PrAgentHostedActionStatus::Failed => "failed",
    }
}

fn readiness_status_label(status: PrAgentReadinessStatus) -> &'static str {
    match status {
        PrAgentReadinessStatus::Ready => "ready",
        PrAgentReadinessStatus::Waiting => "waiting",
        PrAgentReadinessStatus::Blocked => "blocked",
        PrAgentReadinessStatus::Merged => "merged",
        PrAgentReadinessStatus::Stopped => "stopped",
    }
}

fn readiness_action_status_label(status: PrAgentReadinessActionStatus) -> &'static str {
    match status {
        PrAgentReadinessActionStatus::Planned => "planned",
        PrAgentReadinessActionStatus::Applied => "applied",
        PrAgentReadinessActionStatus::Skipped => "skipped",
        PrAgentReadinessActionStatus::Failed => "failed",
    }
}

fn pr_agent_action_priority_label(priority: codex_dev_core::PrAgentActionPriority) -> &'static str {
    match priority {
        codex_dev_core::PrAgentActionPriority::Blocked => "blocked",
        codex_dev_core::PrAgentActionPriority::Required => "required",
        codex_dev_core::PrAgentActionPriority::Wait => "wait",
        codex_dev_core::PrAgentActionPriority::Ready => "ready",
        codex_dev_core::PrAgentActionPriority::Info => "info",
    }
}

fn readiness_check_label(check: &codex_dev_core::PrAgentReadinessCheck) -> String {
    let conclusion = check.conclusion.as_deref().unwrap_or("unknown");
    format!("{} {} / {}", check.name, check.status, conclusion)
}

fn diagnostic_counts(diagnostics: &[codex_dev_core::PrAgentDiagnostic]) -> String {
    if diagnostics.is_empty() {
        return "none".to_string();
    }
    let mut counts: BTreeMap<&'static str, usize> = BTreeMap::new();
    for diagnostic in diagnostics {
        *counts
            .entry(match diagnostic.severity {
                PrAgentSeverity::Info => "info",
                PrAgentSeverity::Warning => "warning",
                PrAgentSeverity::Error => "error",
            })
            .or_default() += 1;
    }
    counts
        .into_iter()
        .map(|(severity, count)| format!("{severity}={count}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn sanitize_list(items: &[String], capsule_path: &Path) -> String {
    items
        .iter()
        .map(|item| redact_path_text(item, capsule_path))
        .collect::<Vec<_>>()
        .join(", ")
}

fn append_artifact_diagnostics(lines: &mut Vec<String>, state: &WorkbenchState) {
    if state.diagnostics.is_empty() {
        return;
    }
    lines.push("artifact diagnostics:".to_string());
    lines.extend(
        state
            .diagnostics
            .iter()
            .map(|diagnostic| format!("- {}", redact_path_text(diagnostic, &state.capsule_path))),
    );
}

fn append_operator_diagnostics(
    lines: &mut Vec<String>,
    report: &TuiOperatorPanelsReport,
    capsule_path: &Path,
) {
    if report.diagnostics.is_empty() {
        return;
    }
    lines.push("operator diagnostics:".to_string());
    lines.extend(
        report
            .diagnostics
            .iter()
            .map(|diagnostic| format!("- {}", redact_path_text(diagnostic, capsule_path))),
    );
}

fn orchestration_severity_label(severity: OrchestrationDiagnosticSeverity) -> &'static str {
    match severity {
        OrchestrationDiagnosticSeverity::Error => "error",
        OrchestrationDiagnosticSeverity::Warning => "warning",
    }
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
        EvidenceSummary, GateRecord, InitArgs, POLICY_GATES_SCHEMA, PR_AGENT_READINESS_SCHEMA,
        PR_AGENT_STATE_SCHEMA, PR_SCHEMA, PolicyGate, PolicyManifest, PolicyProfile, PrAgentAction,
        PrAgentActionPriority, PrAgentDiagnostic, PrAgentHostedActionSpec, PrAgentReadinessAction,
        PrAgentReadinessAttempt, PrAgentReadinessCheck, PrAgentSourceRecord, PrAgentSourceStatus,
        ReviewThreadSummary, SUBAGENTS_SCHEMA, SubagentBatch, SubagentRecord,
        SubagentSynthesisRecord, VERIFICATION_SCHEMA, init_capsule,
    };
    use tempfile::tempdir;

    #[test]
    fn cli_artifact_generation_uses_tui_command_contract() {
        let completion = render_completion(Shell::Zsh).expect("completion");
        assert!(completion.contains("codex-dev-tui"));
        assert!(completion.contains("render-once"));

        let manpage = render_manpage().expect("manpage");
        assert!(manpage.contains("codex-dev-tui"));
        assert!(manpage.contains("Terminal workbench"));
    }

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
        assert_eq!(state.active_panel, Panel::Evidence);
        state.next_panel();
        assert_eq!(state.active_panel, Panel::Subagents);
        state.next_panel();
        assert_eq!(state.active_panel, Panel::Orchestration);
        state.previous_panel();
        assert_eq!(state.active_panel, Panel::Subagents);
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
        let temp = tempdir_in_repo().expect("tempdir");
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
        assert_eq!(state.dashboard.operator.schema, TUI_OPERATOR_PANELS_SCHEMA);
        assert_eq!(
            state
                .dashboard
                .operator
                .task_index
                .as_ref()
                .expect("task index")
                .total,
            2
        );
        assert!(state.dashboard.operator.skill_inventory.is_some());
        assert!(screen.contains("codex-dev dashboard"));
        assert!(screen.contains("skill health:"));
        assert!(screen.contains("task index:"));
        assert!(screen.contains("Dashboard one"));
        assert!(screen.contains("Dashboard two"));
        assert!(screen.contains("subagents:"));
        assert!(screen.contains("pr:"));
    }

    #[test]
    fn dashboard_repo_root_follows_explicit_task_root() {
        let repo_root = repo_root();
        let discovered = dashboard_repo_root(&repo_root.join(".codex/tasks")).expect("repo root");

        assert_eq!(
            discovered,
            fs::canonicalize(repo_root).expect("canonical repo")
        );
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
        assert!(screen.contains("task root does not exist"));
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
    fn evidence_panel_summarizes_records_without_raw_commands() {
        let mut state = fixture_state();
        state.active_panel = Panel::Evidence;

        let screen = render_to_string(&state, 120, 32).expect("render");

        assert!(screen.contains("loaded records: 2"));
        assert!(screen.contains("decision"));
        assert!(screen.contains("research"));
        assert!(screen.contains("sources: decision:panel, docs:pr-agent"));
        assert!(screen.contains("artifacts: <capsule>/pr-agent-state.json"));
        assert!(!screen.contains("gh api repos"));
    }

    #[test]
    fn subagents_panel_summarizes_batches_sources_and_synthesis() {
        let mut state = fixture_state();
        state.active_panel = Panel::Subagents;

        let screen = render_to_string(&state, 120, 34).expect("render");

        assert!(screen.contains("batch pre-pr-review"));
        assert!(screen.contains("runtime_bug_reviewer"));
        assert!(screen.contains("sources: subagent:runtime"));
        assert!(screen.contains("synthesis [accepted; human_verified=true]"));
    }

    #[test]
    fn subagents_panel_redacts_registry_issues() {
        let mut state = fixture_state();
        state.active_panel = Panel::Subagents;
        state
            .subagents
            .as_mut()
            .expect("subagents")
            .batches
            .get_mut(0)
            .expect("batch")
            .registry_issues
            .push("/tmp/tui-fixture/private-plan.json".to_string());

        let screen = render_to_string(&state, 120, 34).expect("render");

        assert!(screen.contains("registry issues: <capsule>/private-plan.json"));
        assert!(!screen.contains("/tmp/tui-fixture/private-plan.json"));
    }

    #[test]
    fn orchestration_panel_renders_completion_and_wait_metadata() {
        let mut state = fixture_state();
        state.active_panel = Panel::Orchestration;

        let screen = render_to_string(&state, 140, 36).expect("render");

        assert!(screen.contains("tui_operator_panels.v1"));
        assert!(screen.contains("batch pre-pr-review"));
        assert!(screen.contains("complete=true"));
        assert!(screen.contains("runtime_bug_reviewer"));
        assert!(screen.contains("wait=completed"));
        assert!(screen.contains("id=agent-runtime-1"));
    }

    #[test]
    fn next_actions_panel_exports_commands_without_raw_outputs() {
        let mut state = fixture_state();
        state.active_panel = Panel::NextActions;

        let screen = render_to_string(&state, 150, 36).expect("render");

        assert!(screen.contains("pr-readiness:merge"));
        assert!(screen.contains("orchestration:pre-pr-review:missing_synthesis"));
        assert!(screen.contains("command: codex-dev pr readiness --capsule CAPSULE_DIR"));
        assert!(screen.contains("--merge --apply"));
        assert!(
            screen.contains("command: codex-dev --json orchestration close --capsule CAPSULE_DIR")
        );
        assert!(
            orchestration_record_command("batch", "reviewer").contains("--disposition accepted")
        );
        assert!(!screen.contains("/tmp/tui-fixture"));
        assert!(!screen.contains("--capsule <capsule>"));
        assert!(!screen.contains("<summary>"));
        assert!(!screen.contains("raw hosted stdout"));
    }

    #[test]
    fn next_action_generation_avoids_body_bearing_commands() {
        let checked_at = Utc.with_ymd_and_hms(2026, 5, 9, 7, 0, 0).unwrap();
        let pr = fixture_pr(checked_at);
        let mut readiness = fixture_pr_readiness(checked_at, pr);
        readiness.actions[0].command = vec![
            "gh".to_string(),
            "pr".to_string(),
            "merge".to_string(),
            "28".to_string(),
            "--body".to_string(),
            "PRIVATE_MERGE_BODY".to_string(),
        ];
        let mut hosted_action = fixture_pr_agent_action(checked_at);
        hosted_action.action.command = vec![
            "gh".to_string(),
            "api".to_string(),
            "-f".to_string(),
            "body=PRIVATE_REVIEW_REPLY".to_string(),
        ];

        let report = capsule_operator_report(
            Path::new("/tmp/private-capsule"),
            None,
            None,
            Some(&readiness),
            &[hosted_action],
        );
        let commands = report
            .next_actions
            .iter()
            .filter_map(|action| action.command.as_deref())
            .collect::<Vec<_>>()
            .join("\n");

        assert!(commands.contains("codex-dev pr readiness --capsule CAPSULE_DIR"));
        assert!(commands.contains("codex-dev pr agent-action --capsule CAPSULE_DIR"));
        assert!(commands.contains("--thread-id PRRT_example"));
        assert!(!commands.contains("PRIVATE_MERGE_BODY"));
        assert!(!commands.contains("PRIVATE_REVIEW_REPLY"));
        assert!(!commands.contains("/tmp/private-capsule"));
    }

    #[test]
    fn hosted_action_next_action_commands_include_required_flags() {
        let checked_at = Utc.with_ymd_and_hms(2026, 5, 9, 7, 0, 0).unwrap();
        let base = fixture_pr_agent_action(checked_at);
        let cases = [
            (
                "post-issue-comment",
                "issue-comment:28",
                "--body-file BODY.md",
            ),
            (
                "reply-review-comment",
                "review-comment:123456",
                "--review-comment-id 123456 --body-file BODY.md",
            ),
            (
                "resolve-review-thread",
                "review-thread:PRRT_123",
                "--thread-id PRRT_123",
            ),
            (
                "unresolve-review-thread",
                "review-thread:PRRT_456",
                "--thread-id PRRT_456",
            ),
            (
                "add-labels",
                "labels:ready,reviewed",
                "--label ready --label reviewed",
            ),
            ("remove-labels", "labels:blocked", "--label blocked"),
            ("rerun-failed-jobs", "workflow-run:98765", "--run-id 98765"),
        ];

        for (kind, target, expected) in cases {
            let mut report = fixture_pr_agent_action(checked_at);
            report.action = base.action.clone();
            report.action.kind = kind.to_string();
            report.action.target = target.to_string();
            let command = hosted_action_next_action_command(&report).expect("command");

            assert!(
                command.contains(expected),
                "{kind} command missing expected flags: {command}"
            );
            assert!(command.contains("--apply"));
            assert!(!command.contains("PRIVATE"));
            assert!(!command.contains('<'));
            assert!(!command.contains('>'));
        }
    }

    #[test]
    fn pr_agent_panel_distinguishes_dry_run_plans_from_ready_actions() {
        let mut state = fixture_state();
        state.active_panel = Panel::PrAgent;

        let screen = render_to_string(&state, 130, 40).expect("render");

        assert!(screen.contains("state report: BjornMelin/dev-skills#28"));
        assert!(screen.contains("readiness: waiting"));
        assert!(screen.contains("merge merge [planned]"));
        assert!(screen.contains("hosted action plans: 1"));
        assert!(screen.contains("dry-run plan"));
        assert!(!screen.contains("raw hosted stdout"));
    }

    #[test]
    fn optional_pr_agent_artifact_parse_errors_render_as_diagnostics() {
        let temp = tempdir().expect("tempdir");
        let root = temp.path().join("tasks");
        let capsule = init_test_capsule(
            &root,
            "partial-pr-agent",
            "Partial PR agent",
            CapsuleStatus::InReview,
            0,
        );
        fs::write(capsule.path.join("pr-agent-state.json"), "{not valid json")
            .expect("write invalid optional artifact");

        let mut state = WorkbenchState::load(&capsule.path).expect("state");
        state.active_panel = Panel::PrAgent;
        let screen = render_to_string(&state, 120, 32).expect("render");

        assert!(state.validation.valid);
        assert!(screen.contains("artifact diagnostics:"));
        assert!(screen.contains("pr-agent-state.json"));
        assert!(screen.contains("<capsule>"));
        assert!(!screen.contains(&capsule.path.display().to_string()));
    }

    #[test]
    fn optional_pr_agent_schema_mismatches_render_as_diagnostics() {
        let temp = tempdir().expect("tempdir");
        let root = temp.path().join("tasks");
        let capsule = init_test_capsule(
            &root,
            "stale-pr-agent-schema",
            "Stale PR agent schema",
            CapsuleStatus::InReview,
            0,
        );
        let checked_at = Utc.with_ymd_and_hms(2026, 5, 9, 7, 0, 0).unwrap();
        let pr = fixture_pr(checked_at);
        let mut state_report = fixture_pr_agent_state(checked_at, pr.clone());
        state_report.schema = "codex-dev.pr-agent-state.v0".to_string();
        let mut readiness_report = fixture_pr_readiness(checked_at, pr);
        readiness_report.schema = "codex-dev.pr-agent-readiness.v0".to_string();
        let action_dir = capsule.path.join("pr-agent-actions").join("stale-plan");
        fs::create_dir_all(&action_dir).expect("action dir");
        let mut action_report = fixture_pr_agent_action(checked_at);
        action_report.schema = "codex-dev.pr-agent-hosted-action.v0".to_string();
        fs::write(
            capsule.path.join("pr-agent-state.json"),
            serde_json::to_string_pretty(&state_report).expect("state json"),
        )
        .expect("write state");
        fs::write(
            capsule.path.join("pr-readiness.json"),
            serde_json::to_string_pretty(&readiness_report).expect("readiness json"),
        )
        .expect("write readiness");
        fs::write(
            action_dir.join("plan.json"),
            serde_json::to_string_pretty(&action_report).expect("action json"),
        )
        .expect("write action");

        let mut state = WorkbenchState::load(&capsule.path).expect("state");
        state.active_panel = Panel::PrAgent;
        let screen = render_to_string(&state, 140, 36).expect("render");

        assert!(state.validation.valid);
        assert!(state.pr_agent_state.is_none());
        assert!(state.pr_readiness.is_none());
        assert!(state.pr_agent_actions.is_empty());
        assert!(screen.contains("pr-agent-state.json schema must be"));
        assert!(screen.contains("pr-readiness.json schema must be"));
        assert!(screen.contains("plan.json schema must be"));
    }

    #[cfg(unix)]
    #[test]
    fn optional_pr_agent_symlinks_render_as_diagnostics() {
        let temp = tempdir().expect("tempdir");
        let root = temp.path().join("tasks");
        let capsule = init_test_capsule(
            &root,
            "symlink-pr-agent-artifacts",
            "Symlink PR agent artifacts",
            CapsuleStatus::InReview,
            0,
        );
        let checked_at = Utc.with_ymd_and_hms(2026, 5, 9, 7, 0, 0).unwrap();
        let external_state_path = temp.path().join("external-pr-agent-state.json");
        fs::write(
            &external_state_path,
            serde_json::to_string_pretty(&fixture_pr_agent_state(
                checked_at,
                fixture_pr(checked_at),
            ))
            .expect("state json"),
        )
        .expect("write external state");
        symlink_path(
            &external_state_path,
            &capsule.path.join("pr-agent-state.json"),
        );

        let action_dir = capsule.path.join("pr-agent-actions").join("symlink-plan");
        fs::create_dir_all(&action_dir).expect("action dir");
        let external_plan_path = temp.path().join("external-plan.json");
        fs::write(
            &external_plan_path,
            serde_json::to_string_pretty(&fixture_pr_agent_action(checked_at))
                .expect("action json"),
        )
        .expect("write external plan");
        symlink_path(&external_plan_path, &action_dir.join("plan.json"));

        let mut state = WorkbenchState::load(&capsule.path).expect("state");
        state.active_panel = Panel::PrAgent;
        let screen = render_to_string(&state, 140, 36).expect("render");

        assert!(state.validation.valid);
        assert!(state.pr_agent_state.is_none());
        assert!(state.pr_agent_actions.is_empty());
        assert!(screen.contains("pr-agent-state.json: symlinks are not supported"));
        assert!(
            screen.contains("pr-agent-actions/symlink-plan/plan.json: symlinks are not supported")
        );
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_pr_agent_actions_root_renders_as_diagnostic() {
        let temp = tempdir().expect("tempdir");
        let root = temp.path().join("tasks");
        let capsule = init_test_capsule(
            &root,
            "symlink-pr-agent-actions-root",
            "Symlink PR agent actions root",
            CapsuleStatus::InReview,
            0,
        );
        let external_actions = temp.path().join("external-pr-agent-actions");
        fs::create_dir_all(&external_actions).expect("external actions");
        symlink_path(&external_actions, &capsule.path.join("pr-agent-actions"));

        let mut state = WorkbenchState::load(&capsule.path).expect("state");
        state.active_panel = Panel::PrAgent;
        let screen = render_to_string(&state, 140, 36).expect("render");

        assert!(state.validation.valid);
        assert!(state.pr_agent_actions.is_empty());
        assert!(screen.contains("pr-agent-actions: symlinks are not supported"));
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

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("repo root")
            .to_path_buf()
    }

    fn tempdir_in_repo() -> std::io::Result<tempfile::TempDir> {
        let parent = repo_root().join("target/tui-tests");
        fs::create_dir_all(&parent)?;
        tempfile::Builder::new()
            .prefix("codex-dev-tui-")
            .tempdir_in(parent)
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
        let pr = fixture_pr(checked_at);
        let capsule_path = PathBuf::from("/tmp/tui-fixture");
        let subagents = fixture_subagents(checked_at);
        let pr_agent_state = fixture_pr_agent_state(checked_at, pr.clone());
        let pr_readiness = fixture_pr_readiness(checked_at, pr.clone());
        let pr_agent_actions = vec![fixture_pr_agent_action(checked_at)];
        let operator = TuiOperatorPanelsReport {
            schema: TUI_OPERATOR_PANELS_SCHEMA.to_string(),
            skill_inventory: None,
            task_index: None,
            orchestration: vec![fixture_orchestration_run(checked_at)],
            next_actions: vec![
                TuiNextActionExport {
                    source: "pr-readiness".to_string(),
                    id: "merge".to_string(),
                    priority: "planned".to_string(),
                    summary: "Merge after readiness turns green".to_string(),
                    command: Some(
                        "codex-dev pr readiness --capsule CAPSULE_DIR --repo BjornMelin/dev-skills --number 28 --merge --apply"
                            .to_string(),
                    ),
                },
                TuiNextActionExport {
                    source: "orchestration".to_string(),
                    id: "pre-pr-review:missing_synthesis".to_string(),
                    priority: "required".to_string(),
                    summary: "Record parent synthesis".to_string(),
                    command: Some(orchestration_close_command("pre-pr-review")),
                },
            ],
            diagnostics: Vec::new(),
        };
        WorkbenchState {
            capsule_path: capsule_path.clone(),
            validation: ValidationResult {
                path: capsule_path.clone(),
                valid: true,
                errors: Vec::new(),
            },
            capsule: Some(StatusResult {
                path: capsule_path,
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
                    total: 2,
                    by_kind: Vec::new(),
                },
            }),
            evidence: fixture_evidence(checked_at),
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
            subagents: Some(subagents),
            pr: Some(pr.clone()),
            pr_agent_state: Some(pr_agent_state),
            pr_readiness: Some(pr_readiness),
            pr_agent_actions,
            operator,
            diagnostics: Vec::new(),
            active_panel: Panel::Pr,
            last_error: None,
        }
    }

    fn fixture_pr(checked_at: chrono::DateTime<Utc>) -> PrEvidence {
        PrEvidence {
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
        }
    }

    fn fixture_evidence(checked_at: chrono::DateTime<Utc>) -> Vec<EvidenceRecord> {
        vec![
            EvidenceRecord {
                schema: codex_dev_core::EVIDENCE_SCHEMA.to_string(),
                kind: EvidenceKind::Decision,
                at: checked_at,
                summary: "Use typed PR-agent artifacts as TUI input".to_string(),
                command: Some("gh api repos/BjornMelin/dev-skills/pulls/28/raw".to_string()),
                exit_code: Some(0),
                source_ids: vec!["decision:panel".to_string(), "docs:pr-agent".to_string()],
                actor: Some("codex".to_string()),
                tool: Some("codex-dev".to_string()),
                confidence: Some(95),
                residual_risk: None,
                artifacts: vec!["/tmp/tui-fixture/pr-agent-state.json".to_string()],
            },
            EvidenceRecord {
                schema: codex_dev_core::EVIDENCE_SCHEMA.to_string(),
                kind: EvidenceKind::Research,
                at: checked_at,
                summary: "Checked current capsule contract docs".to_string(),
                command: None,
                exit_code: None,
                source_ids: vec!["docs:codex-dev-cli".to_string()],
                actor: Some("codex".to_string()),
                tool: Some("codex-dev".to_string()),
                confidence: Some(90),
                residual_risk: Some("fixture only".to_string()),
                artifacts: vec!["docs/reference/codex-dev-cli.md".to_string()],
            },
        ]
    }

    fn fixture_subagents(checked_at: chrono::DateTime<Utc>) -> Subagents {
        Subagents {
            schema: SUBAGENTS_SCHEMA.to_string(),
            batches: vec![SubagentBatch {
                id: "pre-pr-review".to_string(),
                status: "completed".to_string(),
                task: Some("Review TUI evidence panels".to_string()),
                mode: Some("read-only".to_string()),
                scope: Some("crates/codex-dev-tui/src/lib.rs".to_string()),
                wait_policy: Some("parent waits".to_string()),
                rendezvous_required: Some(true),
                registry_issues: Vec::new(),
                duplicate_roles_ignored: BTreeMap::new(),
                prompts: Vec::new(),
                agents: vec![SubagentRecord {
                    role: "runtime_bug_reviewer".to_string(),
                    task: "Find render and loading regressions".to_string(),
                    status: "completed".to_string(),
                    agent_id: Some("agent-runtime-1".to_string()),
                    wait_status: Some("completed".to_string()),
                    wait_elapsed_ms: Some(250),
                    summary: "No blocking runtime regressions found".to_string(),
                    prompt_id: Some("prompt-runtime".to_string()),
                    prompt_hash: Some("abc123".to_string()),
                    disposition: Some("accepted".to_string()),
                    human_verified: true,
                    source_ids: vec!["subagent:runtime".to_string()],
                    artifacts: vec!["/tmp/tui-fixture/subagents.json".to_string()],
                    updated_at: Some(checked_at),
                }],
                synthesis: Some(SubagentSynthesisRecord {
                    status: "accepted".to_string(),
                    summary: "Subagent review is clean".to_string(),
                    human_verified: true,
                    source_ids: vec!["subagent:runtime".to_string()],
                    artifacts: vec!["subagents.json".to_string()],
                    updated_at: checked_at,
                }),
                recorded_at: Some(checked_at),
                updated_at: Some(checked_at),
            }],
        }
    }

    fn fixture_orchestration_run(checked_at: chrono::DateTime<Utc>) -> OrchestrationRunReport {
        OrchestrationRunReport {
            schema: codex_dev_core::ORCHESTRATION_RUN_SCHEMA.to_string(),
            capsule: PathBuf::from("/tmp/tui-fixture"),
            batch_id: "pre-pr-review".to_string(),
            status: "completed".to_string(),
            task: Some("Review TUI evidence panels".to_string()),
            mode: Some("read-only".to_string()),
            scope: Some("crates/codex-dev-tui/src/lib.rs".to_string()),
            wait_policy: Some("parent waits".to_string()),
            rendezvous_required: Some(true),
            expected_roles: vec!["runtime_bug_reviewer".to_string()],
            agents: vec![codex_dev_core::OrchestrationAgentReport {
                role: "runtime_bug_reviewer".to_string(),
                agent_id: Some("agent-runtime-1".to_string()),
                status: "completed".to_string(),
                wait_status: Some("completed".to_string()),
                wait_elapsed_ms: Some(250),
                disposition: Some("accepted".to_string()),
                human_verified: true,
                source_ids: vec!["subagent:runtime".to_string()],
                artifacts: vec!["/tmp/tui-fixture/subagents.json".to_string()],
                updated_at: Some(checked_at),
            }],
            completion: codex_dev_core::OrchestrationCompletionReport {
                expected: 1,
                recorded: 1,
                terminal: 1,
                human_verified: 1,
                missing: Vec::new(),
                extra: Vec::new(),
                synthesis_completed: true,
                complete: true,
            },
            synthesis_status: Some("completed".to_string()),
            registry_issues: Vec::new(),
            diagnostics: Vec::new(),
            checked_at,
            stale_after_minutes: ORCHESTRATION_STALE_AFTER_MINUTES,
        }
    }

    fn fixture_pr_agent_state(
        checked_at: chrono::DateTime<Utc>,
        pr: PrEvidence,
    ) -> PrAgentStateReport {
        PrAgentStateReport {
            schema: PR_AGENT_STATE_SCHEMA.to_string(),
            repository: "BjornMelin/dev-skills".to_string(),
            number: 28,
            checked_at,
            dry_run: true,
            pr,
            sources: vec![PrAgentSourceRecord {
                id: "gh-pr-view".to_string(),
                kind: "gh-pr-view".to_string(),
                command: "gh pr view --json ...".to_string(),
                path: "pr-agent-sources/20260509/gh-pr-view.json".to_string(),
                retrieved_at: checked_at,
                exit_code: Some(0),
                status: PrAgentSourceStatus::Captured,
            }],
            diagnostics: vec![PrAgentDiagnostic {
                source: "gh-rate-limit".to_string(),
                severity: PrAgentSeverity::Info,
                message: "rate limit healthy".to_string(),
                command: Some("gh api rate_limit".to_string()),
                exit_code: Some(0),
                at: checked_at,
            }],
            actions: vec![PrAgentAction {
                id: "wait-coderabbit".to_string(),
                priority: PrAgentActionPriority::Wait,
                summary: "Wait for final hosted review state".to_string(),
                reason: "Review threads are not authoritative yet".to_string(),
            }],
        }
    }

    fn fixture_pr_readiness(
        checked_at: chrono::DateTime<Utc>,
        pr: PrEvidence,
    ) -> PrAgentReadinessReport {
        PrAgentReadinessReport {
            schema: PR_AGENT_READINESS_SCHEMA.to_string(),
            repository: "BjornMelin/dev-skills".to_string(),
            number: 28,
            generated_at: checked_at,
            apply_requested: false,
            rerun_failed_requested: false,
            merge_requested: true,
            ready: false,
            final_status: PrAgentReadinessStatus::Waiting,
            attempts: vec![PrAgentReadinessAttempt {
                attempt: 1,
                checked_at,
                status: PrAgentReadinessStatus::Waiting,
                pr,
                blockers: Vec::new(),
                wait_reasons: vec!["CodeRabbit review is still pending".to_string()],
                warnings: vec!["reviewDecision can lag resolved thread state".to_string()],
                failing_checks: Vec::new(),
                pending_checks: vec![PrAgentReadinessCheck {
                    name: "CodeRabbit".to_string(),
                    status: "queued".to_string(),
                    conclusion: None,
                    url: None,
                    run_id: None,
                    diagnostic_command: "gh pr checks 28".to_string(),
                }],
                active_review_comments: 0,
                outdated_review_comments: 1,
                diagnostics: Vec::new(),
            }],
            actions: vec![PrAgentReadinessAction {
                id: "merge".to_string(),
                kind: "merge".to_string(),
                status: PrAgentReadinessActionStatus::Planned,
                reason: "Merge after readiness turns green".to_string(),
                command: vec![
                    "gh".to_string(),
                    "pr".to_string(),
                    "merge".to_string(),
                    "28".to_string(),
                ],
                exit_code: None,
                stdout: Some("raw hosted stdout".to_string()),
                stderr: None,
            }],
            markdown_path: "pr-readiness.md".to_string(),
            report_path: "pr-readiness.json".to_string(),
        }
    }

    fn fixture_pr_agent_action(checked_at: chrono::DateTime<Utc>) -> PrAgentHostedActionReport {
        PrAgentHostedActionReport {
            schema: codex_dev_core::PR_AGENT_HOSTED_ACTION_SCHEMA.to_string(),
            repository: "BjornMelin/dev-skills".to_string(),
            number: 28,
            plan_id: "resolve-stale-thread".to_string(),
            plan_hash: "planhash".to_string(),
            generated_at: checked_at,
            dry_run: true,
            apply_requested: false,
            action_dir: "pr-agent-actions/resolve-stale-thread".to_string(),
            before_state_path: "pr-agent-actions/resolve-stale-thread/before-state.json"
                .to_string(),
            after_state_path: None,
            action: PrAgentHostedActionSpec {
                id: "resolve-stale-thread".to_string(),
                kind: "resolve-review-thread".to_string(),
                summary: "Resolve stale review thread after verification".to_string(),
                reason: "The thread points at outdated code".to_string(),
                target: "PRRT_example".to_string(),
                idempotency_key: "idempotency".to_string(),
                command: vec!["gh".to_string(), "api".to_string(), "graphql".to_string()],
                duplicate_check_command: Vec::new(),
                state_check_command: Vec::new(),
                requires_apply: true,
                network: true,
                secrets: false,
                permission_notes: vec!["Pull requests write".to_string()],
            },
            diagnostics: Vec::new(),
            execution: None,
        }
    }
}
