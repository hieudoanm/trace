# packages/landing

Static landing page for tracers — plain HTML + CSS, no build step.

## Structure

```
index.html    # single-page site (nav, hero, features, demo, how it works, install, footer)
404.html      # GitHub Pages 404 page
styles.css    # all styles; light page, dark terminals
logo.svg      # brand mark (nav, 404)
favicon.svg   # favicon
og-image.png  # Open Graph image (1280x640)
robots.txt    # allow all + sitemap reference
sitemap.xml   # single-URL sitemap for the Pages site
humans.txt    # project/team credits
```

Asset paths are relative, so the same files work locally and under the
GitHub Pages `/tracers/` base path. Open `index.html` directly or serve the
directory:

```bash
python3 -m http.server 8000 -d packages/landing
```

## Deployment

CI deploys this directory to GitHub Pages on pushes to `master` — there is no
build or test step.
