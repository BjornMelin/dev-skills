use crate::*;

pub(crate) fn handle_run(
    command: RunCommand,
    config: &ResearchConfig,
    json_out: bool,
) -> Result<()> {
    match command {
        RunCommand::Init {
            query,
            profile,
            topic,
            out,
        } => {
            if out.exists() {
                bail!(
                    "run file already exists at {}; move it aside or choose a different --out path",
                    out.display()
                );
            }
            let state = ResearchRunState {
                query: query.clone(),
                profile,
                topic,
                status: RunStatus::Open,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                budgets: profile_budget(config, profile),
                spent: ProviderBudgets::default(),
                debits: Vec::new(),
                provider_errors: Vec::new(),
                source_ids: Vec::new(),
            };
            ensure_parent(&out)?;
            fs::write(&out, serde_json::to_vec_pretty(&state)?)?;
            if json_out {
                print_json(&json!({ "run": out, "state": state }))
            } else {
                println!("run: {}", out.display());
                Ok(())
            }
        }
        RunCommand::Status { run } => {
            let state = read_run_state(&run)?;
            let remaining = remaining_budgets(&state);
            let source_count = state.source_ids.len();
            if json_out {
                print_json(
                    &json!({ "run": run, "state": state, "remaining": remaining, "source_count": source_count }),
                )
            } else {
                println!("status: {:?}", state.status);
                println!("profile: {}", state.profile);
                println!("source_count: {}", source_count);
                println!("remaining:");
                print_budgets(&remaining);
                Ok(())
            }
        }
        RunCommand::Debit {
            run,
            provider,
            count,
            note,
        } => {
            let state = debit_run_budget(&run, provider, count, note.as_deref())?;
            if json_out {
                print_json(
                    &json!({ "run": run, "state": state, "remaining": remaining_budgets(&state) }),
                )
            } else {
                println!("debited {} from {}", count, provider_name(provider));
                Ok(())
            }
        }
        RunCommand::Close { run } => {
            let state = close_run_state(&run)?;
            if json_out {
                print_json(&json!({ "run": run, "state": state }))
            } else {
                println!("closed: {}", run.display());
                Ok(())
            }
        }
    }
}
