# gh GraphQL: review threads + resolve

All native `gh` — no external CLI. Replace `OWNER`, `REPO`, `PR` (and pipe through `jq`).

## Capture the head SHA (drift guard)

```bash
HEAD_SHA=$(gh pr view PR --repo OWNER/REPO --json headRefOid -q .headRefOid)
```

## List unresolved review threads → compact worklist

```bash
gh api graphql -F owner=OWNER -F repo=REPO -F pr=PR -f query='
query($owner:String!,$repo:String!,$pr:Int!,$cursor:String){
  repository(owner:$owner,name:$repo){
    pullRequest(number:$pr){
      reviewThreads(first:100, after:$cursor){
        pageInfo{ hasNextPage endCursor }
        nodes{
          id
          isResolved
          isOutdated
          comments(first:1){ nodes{
            path line originalLine diffHunk body
            author{ login }
          }}
        }
      }
    }
  }
}' | jq '[.data.repository.pullRequest.reviewThreads.nodes[]
  | select(.isResolved|not)
  | { id, isOutdated,
      path: .comments.nodes[0].path,
      line: (.comments.nodes[0].line // .comments.nodes[0].originalLine),
      author: .comments.nodes[0].author.login,
      body: .comments.nodes[0].body,
      suggestion: ((.comments.nodes[0].body | capture("```suggestion\n(?<s>[\\s\\S]*?)```").s) // null) }]'
```

Paginate by passing `-F cursor=ENDCURSOR` while `pageInfo.hasNextPage` is true.

## Resolve a thread (only after committed + pushed + verified)

```bash
gh api graphql -F id="THREAD_ID" -f query='
mutation($id:ID!){ resolveReviewThread(input:{threadId:$id}){ thread{ id isResolved } } }'
```

## (Optional) reply before resolving — only when asked or unfixable

```bash
gh api graphql -F tid="THREAD_ID" -F body="Fixed in COMMIT_SHA." -f query='
mutation($tid:ID!,$body:String!){
  addPullRequestReviewThreadReply(input:{pullRequestReviewThreadId:$tid, body:$body}){ comment{ id } }
}'
```

## Re-confirm head before resolving

```bash
NOW_SHA=$(gh pr view PR --repo OWNER/REPO --json headRefOid -q .headRefOid)
# Resolve ONLY if NOW_SHA equals the SHA you just pushed. Otherwise stop (head drift).
```
