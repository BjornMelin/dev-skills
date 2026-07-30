# Signr Reference

Use Signr as the concrete example:

- Next.js App Router app in `apps/web`
- Clerk auth
- local shadcn component source via `apps/web/components.json`
- browser verification through `browser-workbench`

Generalize from this:

- detect the actual web app root instead of assuming `apps/web`
- detect auth provider instead of assuming Clerk
- detect local component conventions before composing new surfaces
