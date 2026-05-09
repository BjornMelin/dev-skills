use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use clap::Parser;
use codex_dev::{
    Capsule, CapsuleStatus, CheckRecord, PrEvidence, StatusResult, ValidationResult, Verification,
    validate_capsule,
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
pub struct Cli {
    #[arg(long, value_name = "CAPSULE_DIR")]
    capsule: PathBuf,
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

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    if cli.render_once {
        let result = render_once_for_cli(&cli.capsule, cli.width, cli.height)?;
        print!("{}", result.output);
        if !result.valid {
            anyhow::bail!("invalid capsule; see render output for validation details");
        }
        return Ok(());
    }

    run_interactive(&cli.capsule, interactive_tick_rate(cli.tick_ms)?)
        .map_err(|error| sanitized_cli_error(error, &cli.capsule))
}

pub fn run_interactive(capsule_path: &Path, tick_rate: Duration) -> Result<()> {
    let mut terminal = ratatui::init();
    let mut restore_guard = RestoreGuard::new(ratatui::restore);
    let mut state = WorkbenchState::load(capsule_path)?;
    let result = run_app(
        &mut terminal,
        &mut state,
        &mut CrosstermEvents { tick_rate },
    );
    restore_guard.restore_now();
    result
}

pub fn run_app<B, E>(
    terminal: &mut Terminal<B>,
    state: &mut WorkbenchState,
    events: &mut E,
) -> Result<()>
where
    B: Backend,
    B::Error: std::error::Error + Send + Sync + 'static,
    E: EventSource,
{
    loop {
        terminal.draw(|frame| render(frame, state))?;
        match events.next_event()? {
            Some(WorkbenchEvent::Quit) => return Ok(()),
            Some(WorkbenchEvent::NextPanel) => state.next_panel(),
            Some(WorkbenchEvent::PreviousPanel) => state.previous_panel(),
            Some(WorkbenchEvent::Refresh) => state.refresh(),
            None => {}
        }
    }
}

pub trait EventSource {
    fn next_event(&mut self) -> Result<Option<WorkbenchEvent>>;
}

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
        KeyCode::Char('r') => Some(WorkbenchEvent::Refresh),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkbenchEvent {
    Quit,
    NextPanel,
    PreviousPanel,
    Refresh,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Debug)]
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
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let validation = validate_capsule(&path)?;
        let (capsule, verification, pr) = if validation.valid {
            (
                Some(read_capsule_status(&path)?),
                read_optional_json(path.join("verification.json"))?,
                read_optional_json(path.join("pr.json"))?,
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

    pub fn refresh(&mut self) {
        let active_panel = self.active_panel;
        match Self::load(&self.capsule_path) {
            Ok(mut next) => {
                next.active_panel = active_panel;
                *self = next;
            }
            Err(error) => {
                self.last_error = Some(redact_path_text(&format!("{error:#}"), &self.capsule_path));
            }
        }
    }

    pub fn next_panel(&mut self) {
        self.active_panel = self.active_panel.next();
    }

    pub fn previous_panel(&mut self) {
        self.active_panel = self.active_panel.previous();
    }
}

fn read_capsule_status(path: &Path) -> Result<StatusResult> {
    let capsule: Capsule = read_required_json(path.join("capsule.json"))?;
    Ok(StatusResult {
        path: path.to_path_buf(),
        id: capsule.id,
        title: capsule.title,
        status: capsule.status,
        objective: capsule.objective,
        branch: capsule.branch,
        base_branch: capsule.base_branch,
        issues: capsule.issues,
        pull_requests: capsule.pull_requests,
        updated_at: capsule.updated_at,
    })
}

fn read_required_json<T>(path: PathBuf) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let content =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_str(&content).with_context(|| format!("failed to parse {}", path.display()))
}

fn read_optional_json<T>(path: PathBuf) -> Result<Option<T>>
where
    T: serde::de::DeserializeOwned,
{
    if !path.is_file() {
        return Ok(None);
    }
    read_required_json(path).map(Some)
}

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
    pub fn new(restore: F) -> Self {
        Self {
            restore,
            armed: true,
        }
    }

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
            "tab/right: next  shift-tab/left: previous  r: refresh  q/esc/ctrl-c: quit  active: {}",
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
            "{}\n\ncapsule: {}\n\nThis workbench reads codex-dev capsule JSON contracts and does not own policy logic.",
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
        format!("unresolved threads: {}", pr.review_threads.unresolved),
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
    "Open a capsule with --capsule <dir>. Use --render-once for deterministic automation. The UI refreshes by rereading codex-dev JSON contract files."
}

fn render_pr_label(pr: &PrEvidence) -> String {
    match (&pr.repository, pr.number) {
        (Some(repository), Some(number)) => format!("{repository}#{number}"),
        (Some(repository), None) => repository.clone(),
        (None, Some(number)) => format!("#{number}"),
        (None, None) => "no PR".to_string(),
    }
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

pub fn render_to_string(state: &WorkbenchState, width: u16, height: u16) -> Result<String> {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend)?;
    terminal.draw(|frame| render(frame, state))?;
    Ok(buffer_to_string(terminal.backend().buffer()))
}

#[derive(Debug, PartialEq, Eq)]
pub struct RenderOnceResult {
    pub output: String,
    pub valid: bool,
}

pub fn render_once(capsule_path: &Path, width: u16, height: u16) -> Result<RenderOnceResult> {
    let state = WorkbenchState::load(capsule_path)?;
    let valid = state.validation.valid;
    let output = render_to_string(&state, width, height)?;
    Ok(RenderOnceResult { output, valid })
}

pub fn render_once_for_cli(
    capsule_path: &Path,
    width: u16,
    height: u16,
) -> Result<RenderOnceResult> {
    render_once(capsule_path, width, height)
        .map_err(|error| sanitized_cli_error(error, capsule_path))
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
    let path = path.display().to_string();
    if path.is_empty() {
        return text.to_string();
    }
    text.replace(&path, "<capsule>")
}

fn sanitized_cli_error(error: anyhow::Error, capsule_path: &Path) -> anyhow::Error {
    anyhow::anyhow!("{}", redact_path_text(&format!("{error:#}"), capsule_path))
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
    use codex_dev::{GateRecord, ReviewThreadSummary};
    use tempfile::tempdir;

    #[test]
    fn load_reads_codex_dev_contracts() {
        let temp = tempdir().expect("tempdir");
        let root = temp.path().join("tasks");
        let output = codex_dev::run_from([
            "codex-dev",
            "--json",
            "capsule",
            "init",
            "--title",
            "TUI smoke",
            "--branch",
            "feat/codex-dev-tui-workbench",
            "--issue",
            "28",
            "--root",
            root.to_str().expect("utf8 root"),
            "--id",
            "tui-smoke",
            "--created-at",
            "2026-05-09T07:00:00Z",
        ])
        .expect("init capsule");
        let payload: serde_json::Value = serde_json::from_str(&output).expect("json output");
        let path = payload["result"]["path"].as_str().expect("path");

        let state = WorkbenchState::load(path).expect("state");

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
            }),
            verification: Some(Verification {
                schema: codex_dev::VERIFICATION_SCHEMA.to_string(),
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
                schema: codex_dev::PR_SCHEMA.to_string(),
                repository: Some("BjornMelin/dev-skills".to_string()),
                number: Some(28),
                url: Some("https://github.com/BjornMelin/dev-skills/pull/28".to_string()),
                state: "open".to_string(),
                checks: vec![CheckRecord {
                    name: "CodeRabbit".to_string(),
                    status: "completed".to_string(),
                    conclusion: Some("success".to_string()),
                    url: None,
                    checked_at,
                }],
                review_threads: ReviewThreadSummary {
                    unresolved: 0,
                    last_checked_at: checked_at,
                },
            }),
            active_panel: Panel::Pr,
            last_error: None,
        }
    }
}
