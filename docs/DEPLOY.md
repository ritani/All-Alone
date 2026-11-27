# Deploying the web build

The web build is hosted on Cloudflare Pages. GitHub Actions builds and checks it,
then uploads the finished `dist/` folder; Cloudflare does not build anything.

| Branch | Role | Address |
|---|---|---|
| `main` | Production | `https://<project>.pages.dev` (or your custom domain) |
| `MVP` | Staging | `https://mvp.<project>.pages.dev` |

The deploy job in `.github/workflows/check.yml` runs only on pushes to those two
branches, only after the `check` job has passed, and only once the repository
variable below is set. Pull requests are checked but never deployed.

## One-time setup (repository owner)

1. **Cloudflare account and project.** In the Cloudflare dashboard, go to
   Workers & Pages, create a Pages project with *Direct Upload* (not "Connect to
   Git"), and name it (for example `all-alone`). Set its production branch to `main`.
2. **API token.** Create a token with the single permission
   *Account / Cloudflare Pages / Edit*, limited to your account.
3. **GitHub settings** (Settings, Secrets and variables, Actions):
   - secret `CLOUDFLARE_API_TOKEN`: the token from step 2
   - secret `CLOUDFLARE_ACCOUNT_ID`: your Cloudflare account id
   - variable `CLOUDFLARE_PAGES_PROJECT`: the project name from step 1
4. Push to `MVP`. When the check passes, staging is deployed. Merge `MVP` into
   `main` by pull request to release to production.

Optional: in GitHub, Settings, Environments, add required reviewers to the
`production` environment to require a manual approval before each production deploy.

## Notes

- Security headers come from `public/_headers`, which Cloudflare Pages applies as is.
  After the first deploy, confirm them with `curl -I` against the site.
- Staging and production are different origins, so saves made on one are not
  visible on the other.
- The service worker is cache-first. After a deploy, returning players get the new
  version on the load after the one that fetched the update.
- To roll back, use "Rollback" on an earlier deployment in the Cloudflare dashboard,
  or revert the commit and let the workflow deploy again.
