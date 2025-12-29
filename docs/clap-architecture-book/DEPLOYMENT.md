# Deployment Guide

This document provides instructions for deploying the Clap Architecture Book to various hosting platforms.

## Overview

The mdbook generates static HTML files that can be deployed to any static hosting service.

## GitHub Pages Deployment

### Method 1: GitHub Actions (Recommended)

Create `.github/workflows/deploy-book.yml`:

```yaml
name: Deploy Clap Architecture Book

on:
  push:
    branches: [main]
    paths:
      - 'docs/clap-architecture-book/**'
      - 'clap-examples/**'
  workflow_dispatch:

permissions:
  contents: read
  pages: write
  id-token: write

concurrency:
  group: "pages"
  cancel-in-progress: false

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-action@stable

      - name: Install mdbook
        run: cargo install mdbook

      - name: Build examples
        run: |
          cd clap-examples
          cargo build --workspace
          cargo test --workspace

      - name: Build book
        run: |
          cd docs/clap-architecture-book
          mdbook build

      - name: Upload artifact
        uses: actions/upload-pages-artifact@v3
        with:
          path: docs/clap-architecture-book/book

  deploy:
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    runs-on: ubuntu-latest
    needs: build
    steps:
      - name: Deploy to GitHub Pages
        id: deployment
        uses: actions/deploy-pages@v4
```

### Method 2: Manual Deployment

1. Build the book locally:
   ```bash
   cd docs/clap-architecture-book
   mdbook build
   ```

2. Push to gh-pages branch:
   ```bash
   git subtree push --prefix docs/clap-architecture-book/book origin gh-pages
   ```

3. Configure repository settings:
   - Go to Settings > Pages
   - Set source to "Deploy from a branch"
   - Select `gh-pages` branch

## Netlify Deployment

### Configuration

Create `netlify.toml` in repository root:

```toml
[build]
  publish = "docs/clap-architecture-book/book"
  command = """
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
    cargo install mdbook
    cd docs/clap-architecture-book && mdbook build
  """

[build.environment]
  RUST_VERSION = "stable"

[[redirects]]
  from = "/*"
  to = "/index.html"
  status = 200
```

### Deployment Steps

1. Connect your GitHub repository to Netlify
2. Configure build settings (or use netlify.toml)
3. Deploy

## Vercel Deployment

### Configuration

Create `vercel.json`:

```json
{
  "buildCommand": "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && source $HOME/.cargo/env && cargo install mdbook && cd docs/clap-architecture-book && mdbook build",
  "outputDirectory": "docs/clap-architecture-book/book",
  "framework": null
}
```

## Docker Deployment

### Dockerfile

```dockerfile
FROM rust:1.70-slim as builder

RUN cargo install mdbook

WORKDIR /book
COPY docs/clap-architecture-book/ ./
RUN mdbook build

FROM nginx:alpine
COPY --from=builder /book/book /usr/share/nginx/html
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
```

### Build and Run

```bash
docker build -t clap-architecture-book .
docker run -p 8080:80 clap-architecture-book
```

## AWS S3 + CloudFront

### Deploy to S3

```bash
# Build book
cd docs/clap-architecture-book
mdbook build

# Sync to S3
aws s3 sync book/ s3://your-bucket-name/ --delete

# Invalidate CloudFront cache
aws cloudfront create-invalidation \
  --distribution-id YOUR_DISTRIBUTION_ID \
  --paths "/*"
```

### S3 Bucket Policy

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Sid": "PublicReadGetObject",
      "Effect": "Allow",
      "Principal": "*",
      "Action": "s3:GetObject",
      "Resource": "arn:aws:s3:::your-bucket-name/*"
    }
  ]
}
```

## Custom Domain Configuration

### GitHub Pages

1. Create `CNAME` file in book directory with your domain
2. Configure DNS:
   - A record: `185.199.108.153` (GitHub Pages IP)
   - CNAME record: `your-repo.github.io`

### Netlify/Vercel

Configure custom domain in respective dashboards and update DNS accordingly.

## Post-Deployment Verification

After deployment, verify:

1. **Homepage loads**: Visit the deployed URL
2. **Navigation works**: Click through chapters
3. **Search functions**: Use the search bar
4. **Code highlighting**: Check syntax highlighting
5. **Print version**: Access `/print.html`
6. **404 page**: Visit a non-existent URL

## Environment-Specific Configuration

### Production

Update `book.toml` for production:

```toml
[output.html]
site-url = "https://your-domain.com/clap-architecture-book/"
```

### Staging

For staging environments:

```toml
[output.html]
site-url = "https://staging.your-domain.com/clap-architecture-book/"
```

## Monitoring and Analytics

### Add Analytics

Add to `book.toml`:

```toml
[output.html]
additional-js = ["analytics.js"]
```

Create `src/analytics.js`:

```javascript
// Google Analytics or similar
window.dataLayer = window.dataLayer || [];
function gtag(){dataLayer.push(arguments);}
gtag('js', new Date());
gtag('config', 'GA_MEASUREMENT_ID');
```

## Security Considerations

1. **HTTPS**: Ensure all deployments use HTTPS
2. **Content Security Policy**: Configure appropriate CSP headers
3. **Dependencies**: Keep mdbook updated for security patches
4. **Access Control**: Use appropriate access controls for staging environments

## Rollback Procedure

1. Identify the last working commit
2. Rebuild from that commit:
   ```bash
   git checkout <commit-hash>
   cd docs/clap-architecture-book
   mdbook build
   ```
3. Deploy the rebuilt version
4. Investigate and fix the issue in a new branch
