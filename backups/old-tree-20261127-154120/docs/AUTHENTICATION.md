# Authentication policy

## Providers

| Provider | Planned flow | Rule |
| --- | --- | --- |
| Microsoft | Official OAuth device or PKCE flow, then official Minecraft entitlement checks | Requires a project-owned Azure application registration. |
| Ely.by | Ely.by OAuth with client ID `alpinoxi-client` | Follow Ely.by provider documentation and token rules. |
| Offline | Local player name only | Clearly labelled; never used to bypass an online-mode server. |

## Non-negotiable rules

- Do not request or store a Microsoft password.
- Do not log device codes, access tokens, refresh tokens or session tokens.
- Store long-lived secrets only through the OS credential store.
- Keep UI/provider code separated so a failed login cannot affect local instance files.
- A skin service must obey the selected account provider’s rules. A local offline skin stays local unless a server-side system explicitly supports it.
