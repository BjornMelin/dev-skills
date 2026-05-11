use crate::*;

pub(crate) fn output_plan(args: PlanArgs, config: &ResearchConfig, json_out: bool) -> Result<()> {
    let plan = build_plan(&args.query, args.profile, TopicKind::General, config);
    if json_out {
        print_json(&plan)
    } else {
        println!("# Research Plan");
        println!();
        println!("query: {}", plan.query);
        println!("profile: {}", plan.profile);
        println!("route order: {}", route_list(&plan.route_order));
        println!("budgets:");
        println!("  codex web: {}", plan.budgets.codex_web_queries);
        println!("  context7: {}", plan.budgets.context7_calls);
        println!("  github: {}", plan.budgets.github_calls);
        println!("  exa: {}", plan.budgets.exa_calls);
        println!("  direct fetches: {}", plan.budgets.direct_fetches);
        println!("  browser fetches: {}", plan.budgets.browser_fetches);
        println!("  firecrawl: {}", plan.budgets.firecrawl_calls);
        println!("rules:");
        for rule in plan.rules {
            println!("- {rule}");
        }
        Ok(())
    }
}

pub(crate) fn output_search_plan(
    args: SearchArgs,
    config: &ResearchConfig,
    json_out: bool,
) -> Result<()> {
    let plan = build_plan(&args.query, args.profile, args.topic, config);
    if json_out {
        print_json(&plan)
    } else {
        println!("Use these routes in order for `{}`:", args.query);
        for (idx, route) in plan.route_order.iter().enumerate() {
            println!("{}. {}", idx + 1, route_name(*route));
        }
        println!();
        println!("Codex web should handle narrow official/current checks first.");
        println!(
            "Exa is reserved for broad semantic discovery or filtered multi-source exploration."
        );
        Ok(())
    }
}
