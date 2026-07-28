# Managing Tech Debt - All Guest Insights

*18 guests, 20 mentions*

---

## Adriel Frederick
*Adriel Frederick*

> "The answer was like, Yo, we got to rebuild it. There was no answer where we couldn't have a product like this. We needed some ability to be able to influence prices so that we could actually run an effective marketplace. The current solution didn't work. It wasn't as operationally flexible as we needed it to be."

**Insight:** When a technical solution lacks the operational flexibility required by the business, a full rebuild is often necessary despite the emotional and resource cost.

**Tactical advice:**
- Evaluate if current technical debt is preventing necessary operational control.
- Be willing to admit when a complex algorithmic approach has failed and pivot to a more flexible architecture.

*Timestamp: 00:32:15*


## Austin Hay
*Austin Hay*

> "The job of a marketing technologist is to think often one to two years down the road about what we're going to need to solve for and design systems in an elegant way, not to break the bank, but to at least be the minimum viable product to actually get there. And a lot of my job, and I think the job of marketing technologists is trying to preserve that future state in the most minimally invasive engineering and resource way possible."

**Insight:** Preventing future technical debt requires architecting systems that are 'minimally invasive' today but scalable for needs 1-2 years out.

**Tactical advice:**
- When setting up tools, ask: 'What happens a year from now if I don't change anything?'
- Implement foundational elements like SSO or proper data schemas early to avoid catastrophic migrations later.

*Timestamp: 00:50:07*


## Camille Fournier
*Camille Fournier*

> "Engineers notoriously, notoriously, notoriously, massively underestimate the migration time for old system to new system and that causes a lot of problems. By the way, you still have to support the old system while you're working on the new system."

**Insight:** Full system rewrites are often traps because teams underestimate migration time and the burden of supporting two systems simultaneously.

**Tactical advice:**
- Account for significant migration time when planning system updates
- Plan for the resource cost of supporting the legacy system during a transition

*Timestamp: 00:16:24*

---

> "Take pieces potentially of the old system, uplift them, make them more scalable, make them easier to work with, clean up the tech debt, but trying to say we're going to just go away. We're going to rewrite, we're going to build something brand new and it's going to solve all our problems, it just very rarely works."

**Insight:** Incremental evolution and targeted tech debt cleanup are more successful than 'big bang' rewrites.

**Tactical advice:**
- Uplift specific APIs or components rather than the whole framework
- Create a staged plan for system evolution

*Timestamp: 00:19:14*


## Casey Winters
*Casey Winters*

> "The idea is that some of the most impactful projects that product teams can work on at scale... are the hardest to measure. And because of that, they just get chronically underfunded... I walk through some examples of a few tactics that work to get around this problem, building custom metrics to show the value, being able to run small tests that prove the worthwhile-ness of the investment."

**Insight:** Securing investment for non-sexy technical improvements requires quantifying their value through custom metrics and small-scale experiments.

**Tactical advice:**
- Build custom metrics to demonstrate the business value of performance or stability
- Run small tests to prove that technical investments will yield long-term results
- Align with engineering and design peers to present a unified front for technical investments

*Timestamp: 24:49*


## Dylan Field
*Dylan Field 2.0*

> "You always have to keep in mind tech debt and there might be, when you're moving slow, systematic reasons for that. How do you make sure that you're not grinding to a halt because things are built the wrong way or you rush to get something out, and you need to go and fix the underlying infrastructure or way that you built it in some form?"

**Insight:** Systematic slowness is often a symptom of technical debt that requires pausing feature work to fix underlying infrastructure.

**Tactical advice:**
- Investigate systematic reasons for slow development pace
- Balance infrastructure fixes with feature development to maintain long-term speed

*Timestamp: 00:11:15*


## Eeke de Milliano
*Eeke de Milliano*

> "Sometimes teams are just getting bogged down by really urgent work. There's too much tech debt. There's too much product debt. Bugs, instability... There's just no way that they're going to be able to focus on the enlightened, bigger, creative stuff if they're just heads-down dealing with incidents all day."

**Insight:** Unaddressed tech and product debt acts as a ceiling on a team's ability to innovate.

**Tactical advice:**
- Diagnose when a team is stuck in a 'hierarchy of needs' trap due to instability
- Prioritize debt reduction to free up headspace for creative work

*Timestamp: 00:24:18*


## Gaurav Misra
*Gaurav Misra*

> "I actually think as a startup your job is to take on technical debt because that is how you operate faster than a bigger company."

**Insight:** Technical debt is a strategic tool for leverage, allowing startups to move faster by deferring non-critical infrastructure work to future hires.

**Tactical advice:**
- Evaluate if a problem can be solved by a future hire (e.g., the 500th engineer) rather than solving it today.
- Monitor the 'interest' paid on debt—if maintenance takes up 80-90% of time, you have run out of technical debt runway.
- Dedicate specific periods (like Q4) to paying down accumulated debt when product cycles slow down.

*Timestamp: 00:20:31*


## Geoff Charles
*Geoff Charles*

> "We don't have a bug backlog. We fix every bug once they're surfaced almost."

**Insight:** Maintain high product quality and velocity by addressing bugs immediately rather than allowing them to accumulate in a backlog.

**Tactical advice:**
- Assign bugs directly to the engineer on call to ensure immediate pain awareness.
- Use a rotational production engineering program to protect core teams from escalations.

*Timestamp: 00:23:13*


## Julia Schottenstein
*Julia Schottenstein*

> "I try to remind the engineers, we would be so lucky to have tech debt because that means people are using the product... what we didn't need at launch was a distributed scheduler with coworkers and RabbitMQ. We just didn't need it because we had no users."

**Insight:** Technical debt is a 'champagne problem' that indicates product usage; avoid over-engineering at launch before demand is proven.

**Tactical advice:**
- Build the simplest, most 'naive' version of a feature first (e.g., a simple for-loop) to validate demand.
- Accept technical debt as a trade-off for getting the product into users' hands faster.

*Timestamp: 00:53:02*


## Keith Coleman & Jay Baxter
*Keith Coleman & Jay Baxter*

> "deleting code is more important than writing it a lot of the time... engineers have a tendency to add these little incremental wins that actually add more of a long-term maintenance cost than is clear... you get forced to do this, by the way, when you have such a small team."

**Insight:** Small teams must prioritize deleting code over adding features to avoid an unsustainable maintenance burden.

**Tactical advice:**
- Audit systems regularly to delete 'incremental wins' that have high long-term maintenance costs
- Aggressively remove 'cruft' to keep the core system manageable by a small number of people

*Timestamp: 01:03:53*


## Maggie Crowley
*Maggie Crowley*

> "Where are your technical hurdles? What are the big pieces of tech debt? What are your engineering and technical teams always harping on that they want to invest in?"

**Insight:** A comprehensive product strategy must account for the technical constraints and maintenance needs identified by engineering.

**Tactical advice:**
- Interview engineering teams to identify critical technical hurdles
- Include technical debt investments as a core part of the product strategy

*Timestamp: 00:37:34*


## Matt Mullenweg
*Matt Mullenweg*

> "Well, that's why I think technical debt is one of the most interesting concepts. There's so many companies as well that maybe have big market caps, but I feel like they might have billions or tens of billions of dollars of technical debt. You can see in the interface or how their products integrate with themselves through things."

**Insight:** Technical debt is often visible to the end-user through fragmented interfaces and poor product integration.

**Tactical advice:**
- Identify technical debt by looking for inconsistencies in the user interface and product silos

*Timestamp: 00:34:09*

---

> "And it's a big focus for us this year, is actually kind of going back to basics, back to core, and improving all of those kind of nooks and crannies of the user experience, and also ruthlessly editing and cutting as much as possible, because we just launched a lot of stuff over the past 21 years that maybe is not as relevant today or it doesn't need to be there."

**Insight:** Managing long-term debt requires 'ruthlessly editing' and removing features that are no longer relevant to the core mission.

**Tactical advice:**
- Perform a 'back to basics' audit to identify and remove features that no longer serve the primary user goal
- Focus on the 'nooks and crannies' of the UX to resolve accumulated friction

*Timestamp: 00:34:52*


## Melanie Perkins
*Melanie Perkins*

> "We were doing a front-end rewrite and we thought it would take about six months... and then it took two years and it was two years of not shipping any product, two years of a product company not being able to ship product."

**Insight:** Major technical rewrites are 'dark tunnels' that can stall product shipping for years but are necessary for long-term scalability.

**Tactical advice:**
- Gamify long-term technical projects (e.g., using a game board with rubber ducks) to maintain team momentum during 'dark' periods
- Accept that foundational rewrites are necessary to enable future features like cross-platform collaboration

*Timestamp: 00:24:14*


## Tomer Cohen
*Tomer Cohen 2.0*

> "We have the maintenance agent when you have a failed build, it will do it for you. In fact, I think we're close to 50% of all those builds being done by the maintenance agent and a QA agent."

**Insight:** AI can significantly reduce engineering 'toil' by automatically diagnosing and fixing failed builds and handling routine QA tasks.

**Tactical advice:**
- Deploy 'Maintenance Agents' to automatically resolve failed software builds
- Use AI agents to pick up and fix bugs directly from Jira tickets

*Timestamp: 00:27:19*


## Upasna Gautam
*Upasna Gautam*

> "One sprint might be high-priority feature development, in another sprint maybe we're focused on medium-priority optimizations and bug fixes. But we know that any time there's a critical incident in production, it also takes critical priority over everything else."

**Insight:** Balancing maintenance and new features requires a flexible sprint model that can shift priority based on production stability.

**Tactical advice:**
- Establish a clear escalation protocol for critical incidents to protect the team's focus
- Rotate sprint focus between new features and optimizations based on current platform health

*Timestamp: 27:45*


## Ebi Atawodi
*Ebi Atawodi*

> "infrastructure is the product. Period. People are like, 'Oh, tech debt.' I'm like, 'Yeah, it's a product debt.' I cannot build a skyscraper on a shaky foundation. So it is your problem too. It's not for the engineer to be barging on the door and be like, 'Oh, there's a problem.'"

**Insight:** Technical debt should be viewed as 'product debt,' making it a core responsibility of the PM rather than just an engineering concern.

**Tactical advice:**
- Include infrastructure and tech debt in your 'Top 10 Problems' list
- Treat foundational stability as a prerequisite for building new features

*Timestamp: 00:55:22*


## Farhan Thawar
*Farhan Thawar*

> "We have a Delete Code Club. We can always almost find a million-plus lines of code to delete, which is insane. ... Everything gets easier, right? Codelets loads faster. It's easier to understand."

**Insight:** Actively incentivize the deletion of redundant code to improve system maintainability, performance, and developer clarity.

**Tactical advice:**
- Create a 'Delete Code Club' or dedicated hack day teams focused solely on removing code
- Provide a manual or guide for engineers on how to identify and safely delete unused code

*Timestamp: 00:48:04*


## Will Larson
*Will Larson*

> "The decision that was done... they needed to do a complete rewrite in order to get there. This is a decision that never works out for anyone... We try to bring the site up and just keeps crashing. And so it basically takes us a month to get it fully functional again."

**Insight:** Full system rewrites are extremely high-risk and rarely succeed as intended, often leading to significant downtime and business instability.

**Tactical advice:**
- Be wary of 'death march' rewrites intended to solve social or architectural problems
- Expect significant debugging periods (e.g., 30 days) when launching major architectural shifts

*Timestamp: 01:04:37*


