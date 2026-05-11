# Account-aware Mobile Handoff

AtomCode can be used by mobile clients such as GitCodeAlira as an execution
surface for repository, pull request, issue, and review tasks. The first
implementation target is a local daemon handoff; the long-term model is an
account relay that lets the same AtomGit identity connect mobile and desktop
without requiring both devices to be on the same network.

## Flow

1. The mobile client authenticates with GitCode/AtomGit using official OAuth.
2. The user opens a repository, pull request, issue, or diff and chooses to
   continue the work in AtomCode.
3. The mobile client sends a structured handoff payload to AtomCode. The payload
   contains task context such as repository, branch, diff summary, comment draft,
   recent actions, and a return URL. It must not include raw OAuth tokens.
4. AtomCode creates a normal persisted session with the handoff context as the
   first user message.
5. The user continues execution in AtomCode. Tool calls, validation, edits, and
   generated output stay in the AtomCode session history.
6. The result can be returned to GitCode/GitCodeAlira through the original return
   URL, a GitCode comment, a pull request, or a future relay callback.

## Local daemon channel

`POST /handoff` is the near-field channel. It is useful when GitCodeAlira and
AtomCode can reach the same machine or a trusted tunnel.

The daemon creates a session instead of immediately running an agent turn. This
keeps the security model consistent: the desktop AtomCode user still controls
tool approvals, destructive operations, workspace paths, and provider settings.

`GET /handoff` is a read-only preview endpoint for QR/manual links. It lets a
mobile client or browser verify that the daemon can receive the pairing/task
envelope without creating a session. A session is created only by
`POST /handoff`.

`GET /auth/status` exposes account identity and capability flags so mobile
clients can show whether OAuth, CodingPlan, daemon handoff, and future relay
handoff are available.

`GET /codingplan/status` exposes a read-only view of CodingPlan readiness:
login state, configured AtomGit providers, last sync time, remote status, and
model availability.

Example handoff request:

```bash
curl -X POST http://127.0.0.1:3000/handoff \
  -H "content-type: application/json" \
  -d '{
    "source": "pull_request",
    "repo": "owner/repo",
    "branch": "main",
    "task": "Continue reviewing this PR and verify the failing check.",
    "title": "Mobile handoff: review PR",
    "mobile_session_id": "gitcode-alira-record-id",
    "return_url": "https://gitcode.com/owner/repo/pulls/42",
    "diff_summary": "Changed auth and daemon files",
    "comment_draft": "Please verify CodingPlan status before merge",
    "recent_actions": ["opened PR", "viewed diff", "drafted reply"]
  }'
```

## Future account relay

The account relay is a hosted service keyed by the user's AtomGit identity. It
should transport task envelopes, not secrets:

- Mobile creates a handoff task under the signed-in account.
- Desktop AtomCode polls or receives events for that account.
- AtomCode claims a task, creates a local session, and records execution status.
- Mobile displays the state and provides a return path to GitCode objects.

The relay should store minimal metadata, support expiration, and provide clear
user-visible audit history. Tokens stay in the official OAuth storage on each
client and are never forwarded through the handoff payload.

## Security boundaries

- Handoff payloads must not include OAuth access tokens, refresh tokens, API
  keys, or provider secrets.
- The mobile client may suggest a working directory, but AtomCode must validate
  it locally.
- Creating a handoff session must not bypass AtomCode's existing permission
  model for shell commands, writes outside the workspace, or destructive edits.
- Account relay support should require explicit user consent on the desktop
  client before tasks are accepted automatically.
